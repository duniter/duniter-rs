//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Define url type

use crate::host::Host;
use durs_common_tools::fatal_error;
use failure::Fail;
use std::net::SocketAddr;
use std::str::FromStr;
use unwrap::unwrap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UrlWithoutScheme {
    host: Host,
    port: Option<u16>,
    path: Option<String>,
}

#[derive(Clone, Copy, Debug, Fail, PartialEq, Eq)]
pub enum UrlWithoutSchemeParseError {
    #[fail(display = "Empty string.")]
    EmptyStr,
    #[fail(display = "Invalid host: {}.", _0)]
    InvalidHost(url::ParseError),
    #[fail(display = "Invalid URL: {}.", _0)]
    InvalidUrl(url::ParseError),
}

impl FromStr for UrlWithoutScheme {
    type Err = UrlWithoutSchemeParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        if source.is_empty() {
            Err(UrlWithoutSchemeParseError::EmptyStr)
        } else {
            let source_parts: Vec<&str> = source.split('/').collect();
            let host_and_port = source_parts[0];
            let host_and_port_len = host_and_port.len();
            let source_parts2: Vec<&str> = host_and_port.split(':').collect();

            let (host_len, port) = if source_parts2.len() >= 2 {
                let may_port_str = unwrap!(source_parts2.last());
                if let Ok(port) = u16::from_str(*may_port_str) {
                    let host_len = host_and_port_len - may_port_str.len() - 1;
                    (host_len, Some(port))
                } else {
                    (host_and_port_len, None)
                }
            } else {
                (host_and_port_len, None)
            };

            let path = if source_parts.len() >= 2 {
                Some(String::from(&source[host_and_port_len..]))
            } else {
                None
            };

            Ok(UrlWithoutScheme {
                host: Host::parse(&host_and_port[..host_len])
                    .map_err(UrlWithoutSchemeParseError::InvalidHost)?,
                port,
                path,
            })
        }
    }
}

impl ToString for UrlWithoutScheme {
    fn to_string(&self) -> String {
        format!(
            "{host}{port}{path}",
            host = self.host,
            port = if let Some(port) = self.port {
                format!(":{}", port)
            } else {
                "".to_owned()
            },
            path = self.path()
        )
    }
}

impl UrlWithoutScheme {
    pub fn path(&self) -> &str {
        if let Some(ref path) = self.path {
            path.as_str()
        } else {
            ""
        }
    }
    pub fn tls(&self) -> bool {
        if let Some(port) = self.port {
            port == 443u16
        } else {
            false
        }
    }
    pub fn to_url_with_scheme(&self, scheme: &str) -> Result<url::Url, url::ParseError> {
        let scheme = if self.tls() {
            format!("{}s", scheme)
        } else {
            scheme.to_owned()
        };

        let url_str = format!(
            "{scheme}://{url_without_scheme}",
            scheme = scheme,
            url_without_scheme = self.to_string()
        );

        url::Url::parse(&url_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Url {
    Url(url::Url),
    UrlWithoutScheme(UrlWithoutScheme),
}

#[derive(Clone, Copy, Debug, Fail, PartialEq, Eq)]
#[fail(
    display = "Invalid URL: {}. Invalid URL without scheme: {}.",
    url_err, url_without_scheme_err
)]
pub struct UrlParseError {
    url_err: url::ParseError,
    url_without_scheme_err: UrlWithoutSchemeParseError,
}

impl FromStr for Url {
    type Err = UrlParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match UrlWithoutScheme::from_str(source) {
            Ok(url_without_scheme) => Ok(Url::UrlWithoutScheme(url_without_scheme)),
            Err(url_without_scheme_err) => match url::Url::parse(source) {
                Ok(url) => Ok(Url::Url(url)),
                Err(url_err) => Err(UrlParseError {
                    url_err,
                    url_without_scheme_err,
                }),
            },
        }
    }
}

impl Url {
    pub fn tls(&self) -> bool {
        match self {
            Url::Url(url) => match url.scheme() {
                "https" | "wss" => true,
                _ => false,
            },
            Url::UrlWithoutScheme(url_without_scheme) => url_without_scheme.tls(),
        }
    }
    pub fn path(&self) -> &str {
        match self {
            Url::Url(url) => url.path(),
            Url::UrlWithoutScheme(url_without_scheme) => url_without_scheme.path(),
        }
    }
    pub fn to_listenable_addr(&self, default_scheme: &str) -> std::io::Result<Vec<SocketAddr>> {
        self.to_listenable_addr_with_default_port(default_scheme, default_port)
    }
    pub fn to_listenable_addr_with_default_port<F>(
        &self,
        default_scheme: &str,
        default_port: F,
    ) -> std::io::Result<Vec<SocketAddr>>
    where
        F: Fn() -> Option<u16>,
    {
        match self {
            Url::Url(url) => Ok(url.socket_addrs(default_port)?),
            Url::UrlWithoutScheme(url_without_scheme) => {
                match url_without_scheme.to_url_with_scheme(default_scheme) {
                    Ok(url) => Ok(url.socket_addrs(default_port)?),
                    Err(e) => fatal_error!("Fail to convert UrlWithoutScheme to Url: {}", e),
                }
            }
        }
    }
}

#[inline]
fn default_port() -> Option<u16> {
    None
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

    fn ip4() -> Ipv4Addr {
        Ipv4Addr::new(91, 121, 157, 13)
    }
    fn ip6() -> Ipv6Addr {
        Ipv6Addr::new(0x2001, 0x41d0, 1, 0xde0d, 0, 0, 0, 1)
    }

    #[test]
    fn parse_url_with_host_only() -> Result<(), url::ParseError> {
        let host = Host::parse("g1.duniter.org")?;

        let expected_url = Url::UrlWithoutScheme(UrlWithoutScheme {
            host,
            port: None,
            path: None,
        });

        assert_eq!(Ok(expected_url.clone()), Url::from_str("g1.duniter.org"));

        assert_eq!(
            vec![
                SocketAddr::V4(SocketAddrV4::new(ip4(), 80)),
                SocketAddr::V6(SocketAddrV6::new(ip6(), 80, 0, 0))
            ],
            expected_url
                .to_listenable_addr("ws")
                .expect("Fail to get to_listenable_addr addr")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_scheme_and_host() -> Result<(), url::ParseError> {
        let url = Url::Url(url::Url::parse("wss://g1.duniter.org")?);

        assert_eq!(
            vec![
                SocketAddr::V4(SocketAddrV4::new(ip4(), 443)),
                SocketAddr::V6(SocketAddrV6::new(ip6(), 443, 0, 0))
            ],
            url.to_listenable_addr("ws")
                .expect("Fail to get to_listenable_addr addr")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_host_and_port() -> Result<(), url::ParseError> {
        let host = Host::parse("g1.duniter.org")?;

        let expected_url = Url::UrlWithoutScheme(UrlWithoutScheme {
            host,
            port: Some(20901u16),
            path: None,
        });

        assert_eq!(
            Ok(expected_url.clone()),
            Url::from_str("g1.duniter.org:20901")
        );

        assert_eq!(
            vec![
                SocketAddr::V4(SocketAddrV4::new(ip4(), 20901)),
                SocketAddr::V6(SocketAddrV6::new(ip6(), 20901, 0, 0))
            ],
            expected_url
                .to_listenable_addr("ws")
                .expect("Fail to get to_listenable_addr addr")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_scheme_and_host_and_port() -> Result<(), url::ParseError> {
        let url = Url::Url(url::Url::parse("ws://g1.duniter.org:20901")?);

        assert_eq!(
            vec![
                SocketAddr::V4(SocketAddrV4::new(ip4(), 20901)),
                SocketAddr::V6(SocketAddrV6::new(ip6(), 20901, 0, 0))
            ],
            url.to_listenable_addr("ws")
                .expect("Fail to get to_listenable_addr addr")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_host_and_path() -> Result<(), url::ParseError> {
        let host = Host::parse("g1.duniter.org")?;

        assert_eq!(
            Ok(Url::UrlWithoutScheme(UrlWithoutScheme {
                host,
                port: None,
                path: Some("/gva/subscriptions".to_owned()),
            })),
            Url::from_str("g1.duniter.org/gva/subscriptions")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_host_and_port_and_path() -> Result<(), url::ParseError> {
        let host = Host::parse("g1.duniter.org")?;

        assert_eq!(
            Ok(Url::UrlWithoutScheme(UrlWithoutScheme {
                host,
                port: Some(20901u16),
                path: Some("/gva/subscriptions".to_owned()),
            })),
            Url::from_str("g1.duniter.org:20901/gva/subscriptions")
        );

        Ok(())
    }

    #[test]
    fn parse_url_with_scheme_and_host_and_path() -> Result<(), url::ParseError> {
        match Url::from_str("ws://g1.duniter.org/gva/subscriptions") {
            Ok(url) => match url {
                Url::Url(_) => {}
                _ => panic!("expected Url::Url, found other variant !"),
            },
            Err(e) => panic!("Fail to parse url: {} !", e),
        }

        Ok(())
    }
}

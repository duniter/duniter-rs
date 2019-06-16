//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Define host type

use url::Host as UrlHost;

pub type Host = UrlHost<String>;

/*#[derive(Clone, Debug, Hash)]
/// Domain name
pub struct DomainName(String);

impl FromStr for DomainName {
    type Err = PestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match NetworkDocsParser::parse(Rule::domain_name_inner, s) {
            Ok(pairs) => Ok(DomainName(String::from(pairs.as_str()))),
            Err(pest_error) => Err(PestError(format!("{}", pest_error))),
        }
    }
}

impl AsRef<str> for DomainName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for DomainName {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}*/

/*#[derive(Clone, Debug, Hash)]
/// Host
pub enum Host {
    /// Domain name
    DomainName(DomainName),
    // Ip address
    Ip(IpAddr),
}

#[derive(Clone, Debug, Fail)]
#[fail(
    display = "Fail to parse host: It's neither a valid ip address nor a valid domain name. IP error: {}. Domain name error: {}.",
    ip_err, domain_err
)]
/// Host parse error
pub struct HostParseError {
    ip_err: AddrParseError,
    domain_err: PestError,
}

impl FromStr for Host {
    type Err = HostParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match IpAddr::from_str(s) {
            Ok(ip_addr) => Ok(Host::Ip(ip_addr)),
            Err(ip_err) => match DomainName::from_str(s) {
                Ok(domain_name) => Ok(Host::DomainName(domain_name)),
                Err(domain_err) => Err(HostParseError { ip_err, domain_err }),
            },
        }
    }
}*/

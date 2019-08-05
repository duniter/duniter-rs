//  Copyright (C) 2017  The Dunitrust Project Developers.
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

//! Module defining the format of network endpoints and how to handle them.

use crate::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
use hex;
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Total size of all fixed size fields of an EndpointV2
pub static ENDPOINTV2_FIXED_SIZE: &'static usize = &9;
/// Maximum number of network features
pub static MAX_NETWORK_FEATURES_COUNT: &'static usize = &2040;
/// Maximum number of api features
pub static MAX_API_FEATURES_COUNT: &'static usize = &2040;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
/// ApiFeatures
pub struct ApiFeatures(pub Vec<u8>);

impl ApiFeatures {
    fn is_empty(&self) -> bool {
        for byte in &self.0 {
            if *byte > 0u8 {
                return false;
            }
        }
        true
    }

    fn to_string(&self) -> String {
        if self.is_empty() {
            String::from("")
        } else {
            let hex_str = hex::encode(self.0.clone());
            if hex_str.len() == 2 {
                format!("0x{} ", &hex_str[1..])
            } else {
                format!("0x{} ", hex_str)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Identifies the API of an endpoint
pub struct ApiName(pub String);

/// Api version
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ApiVersion(pub usize);

/// Api parts
#[derive(Clone, Debug)]
pub struct ApiPart {
    pub name: ApiName,
    pub versions: HashSet<ApiVersion>,
}

impl ApiPart {
    pub fn union_exist(&self, other: &Self) -> bool {
        if self.name == other.name {
            self.versions.intersection(&other.versions).count() > 0
        } else {
            false
        }
    }
    pub fn contains(&self, api_name: &ApiName, api_version: ApiVersion) -> bool {
        if self.name == *api_name {
            self.versions.contains(&api_version)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint v1
pub struct EndpointV1 {
    /// API Name
    pub api: ApiName,
    /// Node unique identifier
    pub node_id: Option<NodeId>,
    /// Public key of the node declaring this endpoint
    pub issuer: PubKey,
    /// NodeFullID hash
    pub hash_full_id: Option<Hash>,
    /// hostname
    pub host: String,
    /// port number
    pub port: usize,
    /// Optional path
    pub path: Option<String>,
    /// Endpoint in raw format (as it appears on the peer card)
    pub raw_endpoint: String,
    /// Accessibility status of this endpoint  (updated regularly)
    pub status: u32,
    /// Timestamp of the last connection attempt to this endpoint
    pub last_check: u64,
}

impl EndpointV1 {
    /// Accessors providing node full identifier
    pub fn node_full_id(&self) -> Option<NodeFullId> {
        match self.node_id {
            Some(node_id) => Some(NodeFullId(node_id, self.issuer)),
            None => None,
        }
    }
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, _supported_ip_v6: bool) -> Option<String> {
        let protocol = match &self.api.0[..] {
            "WS2P" | "WS2PTOR" => "ws",
            _ => "http",
        };
        let tls = match self.port {
            443 => "s",
            _ => "",
        };
        let path = match self.path {
            Some(ref path_string) => path_string.clone(),
            None => String::new(),
        };
        if get_protocol {
            Some(format!(
                "{}{}://{}:{}/{}",
                protocol, tls, self.host, self.port, path
            ))
        } else {
            Some(format!("{}:{}/{}", self.host, self.port, path))
        }
    }
    /// Generate from pest pair
    fn from_pest_pair(
        pair: Pair<Rule>,
        issuer: PubKey,
        status: u32,
        last_check: u64,
    ) -> EndpointV1 {
        let raw_endpoint = String::from(pair.as_str());
        let mut api_name = "";
        let mut node_id = None;
        let mut hash_full_id = None;
        let mut host_str = "";
        let mut port = 0;
        let mut path = None;

        for ep_pair in pair.into_inner() {
            match ep_pair.as_rule() {
                Rule::api_name => api_name = ep_pair.as_str(),
                Rule::node_id => {
                    node_id = Some(NodeId(u32::from_str_radix(ep_pair.as_str(), 16).unwrap()));
                    hash_full_id = match node_id {
                        Some(node_id_) => Some(NodeFullId(node_id_, issuer).sha256()),
                        None => None,
                    };
                }
                Rule::host => host_str = ep_pair.as_str(),
                Rule::port => port = ep_pair.as_str().parse().unwrap(),
                Rule::path_inner => path = Some(String::from(ep_pair.as_str())),
                _ => fatal_error!("unexpected rule: {:?}", ep_pair.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        EndpointV1 {
            issuer,
            api: ApiName(String::from(api_name)),
            node_id,
            hash_full_id,
            host: String::from(host_str),
            port,
            path,
            raw_endpoint,
            status,
            last_check,
        }
    }

    /// parse from ut8 format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        issuer: PubKey,
        status: u32,
        last_check: u64,
    ) -> Result<EndpointV1, TextDocumentParseError> {
        let mut ep_v1_pairs = NetworkDocsParser::parse(Rule::endpoint_v1, raw_endpoint)?;
        Ok(EndpointV1::from_pest_pair(
            ep_v1_pairs.next().unwrap(),
            issuer,
            status,
            last_check,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Network features
pub struct EndpointV2NetworkFeatures(pub Vec<u8>);

impl ToString for EndpointV2NetworkFeatures {
    fn to_string(&self) -> String {
        if self.is_empty() {
            return "".to_owned();
        }
        let mut features_str = Vec::with_capacity(2);
        if self.http() {
            features_str.push("HTTP");
        }
        if self.ws() {
            features_str.push("WS");
        }
        if self.tls() {
            features_str.push("S");
        }
        if self.tor() {
            features_str.push("TOR");
        }
        format!("{} ", features_str.join(" "))
    }
}

impl EndpointV2NetworkFeatures {
    fn is_empty(&self) -> bool {
        for byte in &self.0 {
            if *byte > 0u8 {
                return false;
            }
        }
        true
    }
    /// Network features size
    pub fn size(&self) -> u8 {
        self.0.len() as u8
    }
    /// Convert Self into bytes
    pub fn to_bytes_slice(&self) -> &[u8] {
        &self.0
    }
    /// HTTP feature is enable ?
    pub fn http(&self) -> bool {
        self.0[0] & 0b0000_0001 == 1u8
    }
    /// WS feature is enable ?
    pub fn ws(&self) -> bool {
        self.0[0] & 0b0000_0010 == 2u8
    }
    /// TLS feature is enable ?
    pub fn tls(&self) -> bool {
        self.0[0] & 0b0000_0100 == 4u8
    }
    /// TOR feature is enable ?
    pub fn tor(&self) -> bool {
        self.0[0] & 0b0000_1000 == 8u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint
pub struct Endpoint {
    /// Endpoint content
    pub content: EndpointEnum,
    /// Accessibility status of this endpoint  (updated regularly)
    pub status: u32,
    /// Timestamp of the last connection attempt to this endpoint
    pub last_check: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint v2
pub struct EndpointV2 {
    /// API Name
    pub api: ApiName,
    /// API version
    pub api_version: u16,
    /// Network features
    pub network_features: EndpointV2NetworkFeatures,
    /// API features
    pub api_features: ApiFeatures,
    /// Domain name
    pub domain: Option<String>,
    /// IPv4
    pub ip_v4: Option<Ipv4Addr>,
    /// IPv6
    pub ip_v6: Option<Ipv6Addr>,
    /// port number
    pub port: u16,
    /// Optional path
    pub path: Option<String>,
}

impl ToString for EndpointV2 {
    fn to_string(&self) -> String {
        let domain: String = if let Some(ref domain) = self.domain {
            format!("{} ", domain)
        } else {
            String::from("")
        };
        let ip4: String = if let Some(ip4) = self.ip_v4 {
            format!("{} ", ip4.to_string())
        } else {
            String::from("")
        };
        let ip6: String = if let Some(ip6) = self.ip_v6 {
            format!("[{}] ", ip6.to_string())
        } else {
            String::from("")
        };
        let path = if let Some(ref path) = self.path {
            format!(" {}", path)
        } else {
            "".to_owned()
        };
        format!(
            "{api} {version}{nf}{af}{ip4}{ip6}{domain}{port}{path}",
            api = self.api.0,
            version = if self.api_version > 0 {
                format!("V{} ", self.api_version)
            } else {
                "".to_owned()
            },
            nf = self.network_features.to_string(),
            af = self.api_features.to_string(),
            port = self.port,
            domain = domain,
            ip4 = ip4,
            ip6 = ip6,
            path = path,
        )
    }
}

impl EndpointV2 {
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, supported_ip_v6: bool) -> Option<String> {
        let protocol = match &self.api.0[..] {
            "WS2P" | "WS2PTOR" => "ws",
            _ => "http",
        };

        let tls = match self.port {
            443 => "s",
            _ => "",
        };
        let domain = if let Some(ref domain) = self.domain {
            domain.clone()
        } else if supported_ip_v6 && self.ip_v6.is_some() {
            let ip_v6 = self.ip_v6.unwrap();
            format!("{}", ip_v6)
        } else if self.ip_v4.is_some() {
            let ip_v4 = self.ip_v4.unwrap();
            format!("{}", ip_v4)
        } else {
            println!("DEBUG: endpoint_v2={:?}", self);
            // Unreacheable endpoint
            return None;
        };
        let path = match self.path {
            Some(ref path_string) => path_string.clone(),
            None => String::new(),
        };
        if get_protocol {
            Some(format!(
                "{}{}://{}:{}/{}",
                protocol, tls, domain, self.port, path
            ))
        } else {
            Some(format!("{}:{}/{}", domain, self.port, path))
        }
    }
    /// Generate from pest pair
    pub fn from_pest_pair(pair: Pair<Rule>) -> Result<EndpointV2, AddrParseError> {
        let mut api_str = "";
        let mut api_version = 0;
        let mut network_features = EndpointV2NetworkFeatures(vec![0u8]);
        let mut api_features = ApiFeatures(vec![]);
        let mut ip_v4 = None;
        let mut ip_v6 = None;
        let mut domain = None;
        let mut port = 0;
        let mut path = None;
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::api_name => api_str = field.as_str(),
                Rule::api_version_inner => api_version = field.as_str().parse().unwrap(),
                Rule::http => network_features.0[0] |= 0b_0000_0001,
                Rule::ws => network_features.0[0] |= 0b_0000_0010,
                Rule::tls => network_features.0[0] |= 0b_0000_0100,
                Rule::tor => network_features.0[0] |= 0b_0000_1000,
                Rule::api_features_inner => {
                    api_features = if field.as_str().len() == 1 {
                        ApiFeatures(hex::decode(&format!("0{}", field.as_str())).unwrap())
                    } else {
                        ApiFeatures(hex::decode(field.as_str()).unwrap())
                    };
                }
                Rule::port => port = field.as_str().parse().unwrap(),
                Rule::domain_name_inner => domain = Some(String::from(field.as_str())),
                Rule::ip4_inner => ip_v4 = Some(Ipv4Addr::from_str(field.as_str())?),
                Rule::ip6_inner => ip_v6 = Some(Ipv6Addr::from_str(field.as_str())?),
                Rule::path_inner => path = Some(String::from(field.as_str())),
                _ => fatal_error!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        if network_features.is_empty() {
            network_features = EndpointV2NetworkFeatures(vec![]);
        }

        Ok(EndpointV2 {
            api: ApiName(String::from(api_str)),
            api_version,
            network_features,
            api_features,
            domain,
            ip_v4,
            ip_v6,
            port,
            path,
        })
    }
    /// parse from raw ascii format
    pub fn parse_from_raw(raw_endpoint: &str) -> Result<EndpointEnum, TextDocumentParseError> {
        let mut ep_v2_pairs = NetworkDocsParser::parse(Rule::endpoint_v2, raw_endpoint)?;
        Ok(EndpointEnum::V2(EndpointV2::from_pest_pair(
            ep_v2_pairs.next().unwrap(),
        )?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint
pub enum EndpointEnum {
    /// Endpoint v1
    V1(EndpointV1),
    /// Endpoint v2
    V2(EndpointV2),
}

impl ToString for EndpointEnum {
    fn to_string(&self) -> String {
        match *self {
            EndpointEnum::V1(ref ep) => ep.raw_endpoint.clone(),
            EndpointEnum::V2(ref ep) => ep.to_string(),
        }
    }
}

impl EndpointEnum {
    /// Accessors providing API name
    pub fn api(&self) -> ApiName {
        match *self {
            EndpointEnum::V1(ref ep) => ep.api.clone(),
            EndpointEnum::V2(ref ep) => ep.api.clone(),
        }
    }
    pub fn version(&self) -> ApiVersion {
        match *self {
            EndpointEnum::V1(_) => ApiVersion(1),
            EndpointEnum::V2(_) => ApiVersion(2),
        }
    }
    /// Accessors providing node unique identifier
    pub fn node_uuid(&self) -> Option<NodeId> {
        match *self {
            EndpointEnum::V1(ref ep) => ep.node_id,
            EndpointEnum::V2(ref _ep) => unreachable!(),
        }
    }
    /// Accessors providing node public key
    pub fn pubkey(&self) -> PubKey {
        match *self {
            EndpointEnum::V1(ref ep) => ep.issuer,
            EndpointEnum::V2(ref _ep) => unreachable!(),
        }
    }
    /// Accessors providing port number
    pub fn port(&self) -> usize {
        match *self {
            EndpointEnum::V1(ref ep) => ep.port,
            EndpointEnum::V2(ref ep) => ep.port as usize,
        }
    }
    /// Accessors providing raw format
    pub fn raw(&self) -> String {
        match *self {
            EndpointEnum::V1(ref ep) => ep.raw_endpoint.clone(),
            _ => fatal_error!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing endpoint accessibility status
    pub fn status(&self) -> u32 {
        match *self {
            EndpointEnum::V1(ref ep) => ep.status,
            EndpointEnum::V2(ref _ep) => unreachable!(),
        }
    }
    /// Set status
    pub fn set_status(&mut self, new_status: u32) {
        match *self {
            EndpointEnum::V1(ref mut ep) => ep.status = new_status,
            EndpointEnum::V2(ref _ep) => unreachable!(),
        }
    }
    /// Set last_check
    pub fn set_last_check(&mut self, new_last_check: u64) {
        match *self {
            EndpointEnum::V1(ref mut ep) => ep.last_check = new_last_check,
            EndpointEnum::V2(ref _ep) => unreachable!(),
        }
    }
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, supported_ip_v6: bool) -> Option<String> {
        match *self {
            EndpointEnum::V1(ref ep) => {
                let protocol = match &ep.api.0[..] {
                    "WS2P" | "WS2PTOR" => "ws",
                    _ => "http",
                };
                let tls = match ep.port {
                    443 => "s",
                    _ => "",
                };
                let path = match ep.path {
                    Some(ref path_string) => path_string.clone(),
                    None => String::new(),
                };
                if get_protocol {
                    Some(format!(
                        "{}{}://{}:{}/{}",
                        protocol, tls, ep.host, ep.port, path
                    ))
                } else {
                    Some(format!("{}:{}/{}", ep.host, ep.port, path))
                }
            }
            EndpointEnum::V2(ref ep_v2) => ep_v2.get_url(get_protocol, supported_ip_v6),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{deserialize, serialize};
    use maplit::hashset;

    #[inline]
    fn api_part_1() -> ApiPart {
        ApiPart {
            name: ApiName("api1".to_owned()),
            versions: hashset![ApiVersion(1)],
        }
    }

    #[test]
    fn test_api_part_contains() {
        let api_part = api_part_1();

        assert_eq!(
            true,
            api_part.contains(&ApiName("api1".to_owned()), ApiVersion(1))
        );

        assert_eq!(
            false,
            api_part.contains(&ApiName("api1".to_owned()), ApiVersion(2))
        );

        assert_eq!(
            false,
            api_part.contains(&ApiName("api2".to_owned()), ApiVersion(1))
        );
    }

    #[test]
    fn test_api_part_union_exist() {
        let api_part = api_part_1();

        assert_eq!(
            false,
            api_part.union_exist(&ApiPart {
                name: ApiName("api2".to_owned()),
                versions: hashset![ApiVersion(1)],
            })
        );

        assert_eq!(
            false,
            api_part.union_exist(&ApiPart {
                name: ApiName("api1".to_owned()),
                versions: hashset![ApiVersion(2), ApiVersion(3)],
            })
        );

        assert_eq!(
            true,
            api_part.union_exist(&ApiPart {
                name: ApiName("api1".to_owned()),
                versions: hashset![ApiVersion(1), ApiVersion(2)],
            })
        );
    }

    #[test]
    fn test_network_features() {
        assert_eq!(EndpointV2NetworkFeatures(vec![1u8]).http(), true);
        assert_eq!(EndpointV2NetworkFeatures(vec![2u8]).ws(), true);
        assert_eq!(EndpointV2NetworkFeatures(vec![4u8]).tls(), true);
        assert_eq!(EndpointV2NetworkFeatures(vec![4u8]).tor(), false);
        assert_eq!(EndpointV2NetworkFeatures(vec![8u8]).tls(), false);
        assert_eq!(EndpointV2NetworkFeatures(vec![8u8]).tor(), true);
        assert_eq!(EndpointV2NetworkFeatures(vec![12u8]).tls(), true);
        assert_eq!(EndpointV2NetworkFeatures(vec![12u8]).tor(), true);

        assert_eq!(
            EndpointV2NetworkFeatures(vec![1u8]).to_string().as_str(),
            "HTTP "
        );
        assert_eq!(
            EndpointV2NetworkFeatures(vec![2u8]).to_string().as_str(),
            "WS "
        );
        assert_eq!(
            EndpointV2NetworkFeatures(vec![4u8]).to_string().as_str(),
            "S "
        );
        assert_eq!(
            EndpointV2NetworkFeatures(vec![8u8]).to_string().as_str(),
            "TOR "
        );
        assert_eq!(
            EndpointV2NetworkFeatures(vec![12u8]).to_string().as_str(),
            "S TOR "
        );
    }
    fn test_parse_and_read_endpoint(str_endpoint: &str, endpoint: EndpointV2) {
        assert_eq!(
            EndpointV2::parse_from_raw(str_endpoint),
            Ok(EndpointEnum::V2(endpoint.clone())),
        );
        let binary_endpoint = serialize(&endpoint).expect("Fail to serialize endpoint !");
        let endpoint2: EndpointV2 =
            deserialize(&binary_endpoint).expect("Fail to deserialize endpoint !");
        assert_eq!(endpoint, endpoint2,);
        assert_eq!(str_endpoint, endpoint.to_string());
    }

    #[test]
    fn test_parse_and_read_minimal_endpoint() {
        let str_endpoint = "UNKNOWN_API 8080";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("UNKNOWN_API")),
            api_version: 0,
            network_features: EndpointV2NetworkFeatures(vec![]),
            api_features: ApiFeatures(vec![]),
            ip_v4: None,
            ip_v6: None,
            domain: None,
            port: 8080u16,
            path: None,
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_localhost_endpoint() {
        let str_endpoint = "WS2P localhost 10900";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 0,
            network_features: EndpointV2NetworkFeatures(vec![]),
            api_features: ApiFeatures(vec![]),
            ip_v4: None,
            ip_v6: None,
            domain: Some(String::from("localhost")),
            port: 10900u16,
            path: None,
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint.clone());
        // test get_url()
        assert_eq!(
            endpoint.get_url(true, false),
            Some("ws://localhost:10900/".to_owned())
        );
    }

    #[test]
    fn test_parse_and_read_classic_v1_endpoint() {
        let str_endpoint = "ES_CORE_API g1.data.duniter.fr 443";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("ES_CORE_API")),
            api_version: 0,
            network_features: EndpointV2NetworkFeatures(vec![]),
            api_features: ApiFeatures(vec![]),
            ip_v4: None,
            ip_v6: None,
            domain: Some(String::from("g1.data.duniter.fr")),
            port: 443u16,
            path: None,
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_host() {
        let str_endpoint = "WS2P V2 S 0x7 g1.durs.ifee.fr 443 ws2p";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            domain: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint.clone());
        // test get_url()
        assert_eq!(
            endpoint.get_url(true, false),
            Some("wss://g1.durs.ifee.fr:443/ws2p".to_owned()),
        );
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4() {
        let str_endpoint = "WS2P V2 S 0x7 84.16.72.210 443 ws2p";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            domain: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv6() {
        let str_endpoint = "WS2P V2 S 0x7 [2001:41d0:8:c5aa::1] 443 ws2p";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            domain: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4_and_ip_v6() {
        let str_endpoint = "WS2P V2 S 0x7 5.135.188.170 [2001:41d0:8:c5aa::1] 443 ws2p";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("5.135.188.170").unwrap()),
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            domain: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_all_fields() {
        let str_endpoint =
            "WS2P V2 S 0x7 5.135.188.170 [2001:41d0:8:c5aa::1] g1.dunitrust.org 443 ws2p";
        let endpoint = EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("5.135.188.170").unwrap()),
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            domain: Some(String::from("g1.dunitrust.org")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }
}

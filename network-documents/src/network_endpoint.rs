//  Copyright (C) 2017  The Duniter Project Developers.
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

extern crate duniter_documents;
extern crate dup_crypto;
extern crate serde;

use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
use hex;
use pest::iterators::Pair;
use pest::Parser;
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;
use std::str::FromStr;
use *;

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
                format!("{} ", &hex_str[1..])
            } else {
                format!("{} ", hex_str)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ParseEndpointError
pub enum ParseEndpointError {
    /// VersionNotSupported
    VersionNotSupported(),
    /// WrongV1Format
    WrongV1Format(String),
    /// WrongV2Format (human-readable explanation)
    WrongV2Format(&'static str),
    /// ApiNameTooLong
    ApiNameTooLong(),
    /// ParseIntError
    ParseIntError(ParseIntError),
    /// UnknowNetworkFeature (feature name)
    UnknowNetworkFeature(String),
    /// Maximum number of network features exceeded
    MaxNetworkFeatures(),
    /// Maximum number of api features exceeded
    MaxApiFeatures(),
    /// UnknowApiFeature (feature name)
    UnknowApiFeature(String),
    /// TooHighApiFeature
    TooHighApiFeature(),
    /// IP Parse error
    AddrParseError(AddrParseError),
    /// Pest grammar error
    PestError(String),
}

impl From<ParseIntError> for ParseEndpointError {
    fn from(e: ParseIntError) -> Self {
        ParseEndpointError::ParseIntError(e)
    }
}

impl From<AddrParseError> for ParseEndpointError {
    fn from(e: AddrParseError) -> Self {
        ParseEndpointError::AddrParseError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Identifies the API of an endpoint
pub struct NetworkEndpointApi(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint v1
pub struct EndpointV1 {
    /// API Name
    pub api: NetworkEndpointApi,
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
                _ => panic!("unexpected rule: {:?}", ep_pair.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        EndpointV1 {
            issuer,
            api: NetworkEndpointApi(String::from(api_name)),
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
    ) -> Result<EndpointV1, ParseEndpointError> {
        match NetworkDocsParser::parse(Rule::endpoint_v1, raw_endpoint) {
            Ok(mut ep_v1_pairs) => Ok(EndpointV1::from_pest_pair(
                ep_v1_pairs.next().unwrap(),
                issuer,
                status,
                last_check,
            )),
            Err(pest_error) => Err(ParseEndpointError::PestError(format!("{}", pest_error))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Network features
pub struct EndpointV2NetworkFeatures(pub Vec<u8>);

impl ToString for EndpointV2NetworkFeatures {
    fn to_string(&self) -> String {
        if self.is_empty() {
            return format!("");
        }
        let mut features_str = Vec::with_capacity(2);
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
    /// Parse network features from utf8 string's array
    pub fn from_str_array(
        str_array: &[&str],
    ) -> Result<EndpointV2NetworkFeatures, ParseEndpointError> {
        let mut network_features = 0u8;
        for nf_str in str_array {
            match *nf_str {
                "IP4" => network_features += 1u8,
                "IP6" => network_features += 2u8,
                "TLS" => network_features += 4u8,
                "TOR" => network_features += 8u8,
                &_ => {
                    return Err(ParseEndpointError::UnknowNetworkFeature(String::from(
                        *nf_str,
                    )))
                }
            }
        }
        Ok(EndpointV2NetworkFeatures(vec![network_features]))
    }
    /// Network features size
    pub fn size(&self) -> u8 {
        self.0.len() as u8
    }
    /// Convert Self into bytes
    pub fn to_bytes_slice(&self) -> &[u8] {
        &self.0
    }
    /// network feature ip_v4 is enable ?
    pub fn ip_v4(&self) -> bool {
        self.0[0] & 0b0000_0001 == 1u8
    }
    /// network feature ip_v6 is enable ?
    pub fn ip_v6(&self) -> bool {
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
/// Endpoint v2
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
    pub api: NetworkEndpointApi,
    /// API version
    pub api_version: u16,
    /// Network features
    pub network_features: EndpointV2NetworkFeatures,
    /// API features
    pub api_features: ApiFeatures,
    /// IPv4
    pub ip_v4: Option<Ipv4Addr>,
    /// IPv6
    pub ip_v6: Option<Ipv6Addr>,
    /// hostname
    pub host: Option<String>,
    /// port number
    pub port: u16,
    /// Optional path
    pub path: Option<String>,
}

impl ToString for EndpointV2 {
    fn to_string(&self) -> String {
        let host: String = if let Some(ref host) = self.host {
            format!("{} ", host)
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
            format!("")
        };
        format!(
            "{api} {version}{nf}{af}{host}{ip4}{ip6}{port}{path}",
            api = self.api.0,
            version = if self.api_version > 0 {
                format!("V{} ", self.api_version)
            } else {
                format!("")
            },
            nf = self.network_features.to_string(),
            af = self.api_features.to_string(),
            port = self.port,
            host = host,
            ip4 = ip4,
            ip6 = ip6,
            path = path,
        )
    }
}

impl EndpointV2 {
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, supported_ip_v6: bool) -> Option<String> {
        let protocol = self.api.0.clone();
        let tls = match self.port {
            443 => "s",
            _ => "",
        };
        let host = if let Some(ref host) = self.host {
            host.clone()
        } else if supported_ip_v6 && self.ip_v6.is_some() {
            let ip_v6 = self.ip_v6.unwrap();
            format!("{}", ip_v6)
        } else if self.ip_v4.is_some() {
            let ip_v4 = self.ip_v4.unwrap();
            format!("{}", ip_v4)
        } else {
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
                protocol, tls, host, self.port, path
            ))
        } else {
            Some(format!("{}:{}/{}", host, self.port, path))
        }
    }
    /// Generate from pest pair
    fn from_pest_pair(pair: Pair<Rule>) -> EndpointV2 {
        let mut api_str = "";
        let mut api_version = 0;
        let mut network_features = EndpointV2NetworkFeatures(vec![0u8]);
        let mut api_features = ApiFeatures(vec![]);
        let mut ip_v4 = None;
        let mut ip_v6 = None;
        let mut host = None;
        let mut port = 0;
        let mut path = None;
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::api_name => api_str = field.as_str(),
                Rule::api_version_inner => api_version = field.as_str().parse().unwrap(),
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
                Rule::host_v2_inner => host = Some(String::from(field.as_str())),
                Rule::ip4_inner => ip_v4 = Some(Ipv4Addr::from_str(field.as_str()).unwrap()),
                Rule::ip6_inner => ip_v6 = Some(Ipv6Addr::from_str(field.as_str()).unwrap()),
                Rule::path_inner => path = Some(String::from(field.as_str())),
                _ => panic!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        if network_features.is_empty() {
            network_features = EndpointV2NetworkFeatures(vec![]);
        }
        EndpointV2 {
            api: NetworkEndpointApi(String::from(api_str)),
            api_version,
            network_features,
            api_features,
            ip_v4,
            ip_v6,
            host,
            port,
            path,
        }
    }
    /// parse from ut8 format
    pub fn parse_from_raw(raw_endpoint: &str) -> Result<EndpointEnum, ParseEndpointError> {
        match NetworkDocsParser::parse(Rule::endpoint_v2, raw_endpoint) {
            Ok(mut ep_v2_pairs) => Ok(EndpointEnum::V2(EndpointV2::from_pest_pair(
                ep_v2_pairs.next().unwrap(),
            ))),
            Err(pest_error) => Err(ParseEndpointError::PestError(format!("{}", pest_error))),
        }

        /*let raw_ep_elements: Vec<&str> = raw_endpoint.split(' ').collect();
        if raw_ep_elements.len() >= 6 {
            let api = NetworkEndpointApi(String::from(raw_ep_elements[0]));
            let api_version: u16 = raw_ep_elements[1].parse()?;
            let network_features_count: usize = raw_ep_elements[2].parse()?;
            if network_features_count > *MAX_NETWORK_FEATURES_COUNT {
                Err(ParseEndpointError::MaxNetworkFeatures())
            } else if raw_ep_elements.len() >= 6 + network_features_count {
                let network_features = EndpointV2NetworkFeatures::from_str_array(
                    &raw_ep_elements[3..(3 + network_features_count)],
                )?;
                let api_features_count: usize =
                    raw_ep_elements[3 + network_features_count].parse()?;
                if network_features_count > *MAX_API_FEATURES_COUNT {
                    Err(ParseEndpointError::MaxApiFeatures())
                } else {
                    let mut af_bytes_count = network_features_count / 8;
                    if network_features_count % 8 != 0 {
                        af_bytes_count += 1;
                    }
                    let mut api_features = vec![0u8; af_bytes_count];
                    if raw_ep_elements.len() < 4 + network_features_count + api_features_count {
                        return Err(ParseEndpointError::WrongV2Format(
                            "All api features must be declared !",
                        ));
                    }
                    for str_feature in raw_ep_elements
                        .iter()
                        .take(4 + network_features_count + api_features_count)
                        .skip(4 + network_features_count)
                    {
                        if let Ok(feature) = str_feature.parse::<usize>() {
                            if feature > *MAX_API_FEATURES_COUNT {
                                return Err(ParseEndpointError::TooHighApiFeature());
                            }
                            let byte_index = feature / 8;
                            let feature = (feature % 8) as u8;
                            api_features[byte_index] += feature.pow(2);
                        } else if &api.0 == "WS2P" {
                            match *str_feature {
                                "DEF" => api_features[0] += 1u8,
                                "LOW" => api_features[0] += 2u8,
                                "ABF" => api_features[0] += 4u8,
                                _ => {
                                    return Err(ParseEndpointError::UnknowApiFeature(String::from(
                                        *str_feature,
                                    )))
                                }
                            }
                        } else {
                            return Err(ParseEndpointError::UnknowApiFeature(String::from(
                                *str_feature,
                            )));
                        }
                    }
                    let mut index = 4 + network_features_count + api_features_count;
                    let port = if let Ok(port) = raw_ep_elements[index].parse::<u16>() {
                        index += 1;
                        port
                    } else {
                        return Err(ParseEndpointError::WrongV2Format(
                            "Missing port or is not integer !",
                        ));
                    };
                    // HOST IP4 [IP6] PATH
                    let (host, ip_v4, ip_v6, path) = if raw_ep_elements.len() == index + 4 {
                        // HOST IP4 [IP6] PATH
                        let len2 = raw_ep_elements[index + 2].len();
                        (
                            Some(String::from(raw_ep_elements[index])),
                            Some(Ipv4Addr::from_str(raw_ep_elements[index + 1])?),
                            Some(Ipv6Addr::from_str(
                                &raw_ep_elements[index + 2][1..len2 - 1],
                            )?),
                            Some(String::from(raw_ep_elements[index + 3])),
                        )
                    } else if raw_ep_elements.len() == index + 3 {
                        // IP4 [IP6] PATH
                        if let Ok(ip_v4) = Ipv4Addr::from_str(raw_ep_elements[index]) {
                            let len1 = raw_ep_elements[index + 1].len();
                            (
                                None,
                                Some(ip_v4),
                                Some(Ipv6Addr::from_str(
                                    &raw_ep_elements[index + 1][1..len1 - 1],
                                )?),
                                Some(String::from(raw_ep_elements[index + 2])),
                            )
                        } else {
                            let len1 = raw_ep_elements[index + 1].len();
                            let len2 = raw_ep_elements[index + 2].len();
                            if let Some('[') = raw_ep_elements[index + 1].chars().next() {
                                // HOST [IP6] PATH
                                (
                                    Some(String::from(raw_ep_elements[index])),
                                    None,
                                    Some(Ipv6Addr::from_str(
                                        &raw_ep_elements[index + 1][1..len1 - 1],
                                    )?),
                                    Some(String::from(raw_ep_elements[index + 2])),
                                )
                            } else if let Some('[') = raw_ep_elements[index + 2].chars().next() {
                                // HOST IP4 [IP6]
                                (
                                    Some(String::from(raw_ep_elements[index])),
                                    Some(Ipv4Addr::from_str(raw_ep_elements[index + 1])?),
                                    Some(Ipv6Addr::from_str(
                                        &raw_ep_elements[index + 2][1..len2 - 1],
                                    )?),
                                    None,
                                )
                            } else {
                                // HOST IP4 PATH
                                (
                                    Some(String::from(raw_ep_elements[index])),
                                    Some(Ipv4Addr::from_str(raw_ep_elements[index + 1])?),
                                    None,
                                    Some(String::from(raw_ep_elements[index + 2])),
                                )
                            }
                        }
                    } else if raw_ep_elements.len() == index + 2 {
                        let len0 = raw_ep_elements[index].len();
                        let len1 = raw_ep_elements[index + 1].len();
                        if let Ok(ip_v4) = Ipv4Addr::from_str(raw_ep_elements[index]) {
                            if let Some('[') = raw_ep_elements[index + 1].chars().next() {
                                // IP4 [IP6]
                                (
                                    None,
                                    Some(ip_v4),
                                    Some(Ipv6Addr::from_str(
                                        &raw_ep_elements[index + 1][1..len1 - 1],
                                    )?),
                                    None,
                                )
                            } else {
                                // IP4 PATH
                                (
                                    None,
                                    Some(ip_v4),
                                    None,
                                    Some(String::from(raw_ep_elements[index + 1])),
                                )
                            }
                        } else if let Some('[') = raw_ep_elements[index].chars().next() {
                            // [IP6] PATH
                            (
                                None,
                                None,
                                Some(Ipv6Addr::from_str(&raw_ep_elements[index][1..len0 - 1])?),
                                Some(String::from(raw_ep_elements[index + 1])),
                            )
                        } else if let Ok(ip_v4) = Ipv4Addr::from_str(raw_ep_elements[index + 1]) {
                            // HOST IP4
                            (
                                Some(String::from(raw_ep_elements[index])),
                                Some(ip_v4),
                                None,
                                None,
                            )
                        } else if let Some('[') = raw_ep_elements[index + 1].chars().next() {
                            // HOST [IP6]
                            (
                                Some(String::from(raw_ep_elements[index])),
                                None,
                                Some(Ipv6Addr::from_str(
                                    &raw_ep_elements[index + 1][1..len1 - 1],
                                )?),
                                None,
                            )
                        } else {
                            // HOST PATH
                            (
                                Some(String::from(raw_ep_elements[index])),
                                None,
                                None,
                                Some(String::from(raw_ep_elements[index + 1])),
                            )
                        }
                    } else if raw_ep_elements.len() == index + 1 {
                        let len0 = raw_ep_elements[index].len();
                        if let Some('[') = raw_ep_elements[index].chars().next() {
                            // IP6
                            (
                                None,
                                None,
                                Some(Ipv6Addr::from_str(&raw_ep_elements[index][1..len0])?),
                                None,
                            )
                        } else if let Ok(ip_v4) = Ipv4Addr::from_str(raw_ep_elements[index]) {
                            // IP4
                            (None, Some(ip_v4), None, None)
                        } else {
                            // HOST
                            (Some(String::from(raw_ep_elements[index])), None, None, None)
                        }
                    } else {
                        return Err(ParseEndpointError::WrongV2Format("Invalid fields count !"));
                    };
                    Ok(EndpointEnum::V2(EndpointV2 {
                        api,
                        api_version,
                        network_features,
                        api_features: ApiFeatures(api_features.to_vec()),
                        ip_v4,
                        ip_v6,
                        host,
                        port,
                        path,
                    }))
                }
            } else {
                Err(ParseEndpointError::WrongV2Format(
                    "All network features must be declared !",
                ))
            }
        } else {
            Err(ParseEndpointError::WrongV2Format(
                "An endpoint must contain at least 6 elements",
            ))
        }*/
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
    pub fn api(&self) -> NetworkEndpointApi {
        match *self {
            EndpointEnum::V1(ref ep) => ep.api.clone(),
            EndpointEnum::V2(ref ep) => ep.api.clone(),
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
            _ => panic!("Endpoint version is not supported !"),
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
    use tests::bincode::{deserialize, serialize};

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
            api: NetworkEndpointApi(String::from("UNKNOWN_API")),
            api_version: 0,
            network_features: EndpointV2NetworkFeatures(vec![]),
            api_features: ApiFeatures(vec![]),
            ip_v4: None,
            ip_v6: None,
            host: None,
            port: 8080u16,
            path: None,
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_classic_v1_endpoint() {
        let str_endpoint = "ES_CORE_API g1.data.duniter.fr 443";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("ES_CORE_API")),
            api_version: 0,
            network_features: EndpointV2NetworkFeatures(vec![]),
            api_features: ApiFeatures(vec![]),
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.data.duniter.fr")),
            port: 443u16,
            path: None,
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_host() {
        let str_endpoint = "WS2P V2 S 7 g1.durs.ifee.fr 443 ws2p";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4() {
        let str_endpoint = "WS2P V2 S 7 84.16.72.210 443 ws2p";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv6() {
        let str_endpoint = "WS2P V2 S 7 [2001:41d0:8:c5aa::1] 443 ws2p";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4_and_ip_v6() {
        let str_endpoint = "WS2P V2 S 7 5.135.188.170 [2001:41d0:8:c5aa::1] 443 ws2p";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("5.135.188.170").unwrap()),
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }

    #[test]
    fn test_parse_and_read_endpoint_with_all_fields() {
        let str_endpoint = "WS2P V2 S 7 g1.durs.info 5.135.188.170 [2001:41d0:8:c5aa::1] 443 ws2p";
        let endpoint = EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("5.135.188.170").unwrap()),
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            host: Some(String::from("g1.durs.info")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        };
        test_parse_and_read_endpoint(str_endpoint, endpoint);
    }
}

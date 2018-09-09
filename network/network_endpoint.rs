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

extern crate crypto;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_module;
extern crate regex;
extern crate serde;

use self::regex::Regex;
use duniter_crypto::hashs::Hash;
use duniter_crypto::keys::PubKey;
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;
use std::str::FromStr;
use {ApiFeatures, NodeFullId, NodeId};

/// Total size of all fixed size fields of an EndpointV2
pub static ENDPOINTV2_FIXED_SIZE: &'static usize = &9;
/// Maximum number of network features
pub static MAX_NETWORK_FEATURES_COUNT: &'static usize = &2040;
/// Maximum number of api features
pub static MAX_API_FEATURES_COUNT: &'static usize = &2040;

lazy_static! {
    #[derive(Debug)]
    /// Regex match all endpoint in V1 format (works for all api)
    pub static ref ENDPOINT_V1_REGEX: Regex = Regex::new(
        r"^(?P<api>[A-Z0-9_]+) (?P<version>[1-9][0-9]*)? ?(?P<uuid>[a-f0-9]{6,8})? ?(?P<host>[a-z_][a-z0-9-_.]*|[0-9.]+|[0-9a-f:]+) (?P<port>[0-9]+)(?: /?(?P<path>.+)?)? *$"
    ).unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ParseEndpointError
pub enum ParseEndpointError {
    /// VersionNotSupported
    VersionNotSupported(),
    /// WrongV1Format
    WrongV1Format(),
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

#[derive(Debug)]
/// Error when converting a byte vector to Endpoint
pub enum EndpointReadBytesError {
    /// Bytes vector is too short
    TooShort(),
    /// Bytes vector is too long
    TooLong(),
    /// Wrong api datas Length
    WrongApiDatasLen(),
    /// Unknow api name
    UnknowApiName(),
    /// IoError
    IoError(::std::io::Error),
    /// FromUtf8Error
    FromUtf8Error(::std::string::FromUtf8Error),
}

impl From<::std::io::Error> for EndpointReadBytesError {
    fn from(e: ::std::io::Error) -> Self {
        EndpointReadBytesError::IoError(e)
    }
}

impl From<::std::string::FromUtf8Error> for EndpointReadBytesError {
    fn from(e: ::std::string::FromUtf8Error) -> Self {
        EndpointReadBytesError::FromUtf8Error(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Identifies the API of an endpoint
pub struct NetworkEndpointApi(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint v1
pub struct EndpointEnumV1 {
    /// API version
    pub version: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Network features
pub struct EndpointV2NetworkFeatures(pub Vec<u8>);

impl EndpointV2NetworkFeatures {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Size informations of Endpoint v2
pub struct EndpointV2Size {
    /// Api nalme size
    pub api_size: u8,
    /// Hostname size
    pub host_size: u8,
    /// Optional path size
    pub path_size: u8,
    /// Network features size
    pub nf_size: u8,
    /// Network feature ip_v4
    pub ip_v4: bool,
    /// Network feature ip_v6
    pub ip_v6: bool,
    /// Api features size
    pub af_size: u8,
}

impl EndpointV2Size {
    /// Compute total size of endpoint in binary format
    pub fn total_size(self) -> usize {
        let mut total_size = self.api_size as usize
            + self.host_size as usize
            + self.path_size as usize
            + self.nf_size as usize
            + self.af_size as usize
            + ENDPOINTV2_FIXED_SIZE;
        if self.api_size == 0u8 {
            total_size += 1;
        }
        if self.ip_v4 {
            total_size += 4;
        }
        if self.ip_v6 {
            total_size += 16;
        }
        total_size
    }
}

/*
impl BinMessage for EndpointV2 {
    type ReadBytesError = EndpointReadBytesError;

    fn to_bytes_vector(&self) -> Vec<u8> {
        let endpoint_size = self.compute_endpoint_size();
        let mut binary_endpoint = Vec::with_capacity(endpoint_size.total_size());
        binary_endpoint.push(endpoint_size.api_size);
        binary_endpoint.push(endpoint_size.host_size);
        binary_endpoint.push(endpoint_size.path_size);
        binary_endpoint.append(&mut self.api.to_bytes_vector());
        // api_version
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(self.api_version)
            .expect("Unable to write");
        binary_endpoint.extend_from_slice(&buffer);
        // nf_size
        binary_endpoint.push(endpoint_size.nf_size);
        // network_features
        binary_endpoint.extend_from_slice(&self.network_features.to_bytes_slice());
        binary_endpoint.push(endpoint_size.af_size);
        binary_endpoint.append(&mut self.api_features.0.clone());
        if let Some(ip_v4) = self.ip_v4 {
            binary_endpoint.extend_from_slice(&ip_v4.octets());
        }
        if let Some(ip_v6) = self.ip_v6 {
            binary_endpoint.extend_from_slice(&ip_v6.octets());
        }
        if let Some(ref host) = self.host {
            binary_endpoint.extend_from_slice(host.as_bytes());
        }
        // port
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(self.port)
            .expect("Unable to write");
        binary_endpoint.extend_from_slice(&buffer);
        // path
        if let Some(ref path) = self.path {
            binary_endpoint.extend_from_slice(path.as_bytes());
        }
        binary_endpoint
    }
    /// Create endpoint from bytes vector
    fn from_bytes(binary_ep: &[u8]) -> Result<EndpointV2, EndpointReadBytesError> {
        if binary_ep.len() < *ENDPOINTV2_FIXED_SIZE {
            return Err(EndpointReadBytesError::TooShort());
        }
        let api_size = binary_ep[0] as usize;
        let host_size = binary_ep[1] as usize;
        let path_size = binary_ep[2] as usize;
        if binary_ep.len() < (*ENDPOINTV2_FIXED_SIZE + api_size + host_size + path_size) {
            return Err(EndpointReadBytesError::TooShort());
        }
        let mut index: usize = 3;
        // read api
        let api_datas = if api_size == 0 {
            index += 1;
            &binary_ep[index - 1..index]
        } else {
            index += api_size;
            &binary_ep[index - api_size..index]
        };
        let api = NetworkEndpointApi::api_from_bytes(api_size, api_datas)?;
        // read api_version
        let mut api_version_bytes = Cursor::new(binary_ep[index..index + 2].to_vec());
        index += 2;
        let api_version = api_version_bytes.read_u16::<BigEndian>()?;
        // read nf_size
        let nf_size = binary_ep[index] as usize;
        index += 1;
        if binary_ep.len() < index + nf_size + 1 {
            return Err(EndpointReadBytesError::TooShort());
        }
        // read network_features
        let network_features =
            EndpointV2NetworkFeatures(binary_ep[index..index + nf_size].to_vec());
        index += nf_size;
        // read af_size
        let af_size = binary_ep[index] as usize;
        index += 1;
        if binary_ep.len() < index + af_size + 1 {
            return Err(EndpointReadBytesError::TooShort());
        }
        // read api_features
        let api_features = ApiFeatures(binary_ep[index..index + nf_size].to_vec());
        index += af_size;
        // read ip_v4
        let ip_v4 = network_features.ip_v4();
        if binary_ep.len() < index + 4 && ip_v4 {
            return Err(EndpointReadBytesError::TooShort());
        }
        let ip_v4 = if ip_v4 {
            index += 4;
            Some(Ipv4Addr::new(
                binary_ep[index - 4],
                binary_ep[index - 3],
                binary_ep[index - 2],
                binary_ep[index - 1],
            ))
        } else {
            None
        };
        // read ip_v6
        let ip_v6 = network_features.ip_v6();
        if binary_ep.len() < index + 16 && ip_v6 {
            return Err(EndpointReadBytesError::TooShort());
        }
        let ip_v6 = if ip_v6 {
            index += 16;
            let mut ip_v6_datas: [u8; 16] = [0u8; 16];
            ip_v6_datas.copy_from_slice(&binary_ep[index - 16..index]);
            Some(Ipv6Addr::from(ip_v6_datas))
        } else {
            None
        };
        // read host
        if binary_ep.len() < index + host_size + 2 {
            return Err(EndpointReadBytesError::TooShort());
        }
        let host = if host_size > 0 {
            index += host_size;
            Some(String::from_utf8(
                binary_ep[index - host_size..index].to_vec(),
            )?)
        } else {
            None
        };
        // read port
        let mut port_bytes = Cursor::new((&binary_ep[index..index + 2]).to_vec());
        index += 2;
        let port = port_bytes.read_u16::<BigEndian>()?;
        // read path
        if binary_ep.len() < index + path_size {
            return Err(EndpointReadBytesError::TooShort());
        } else if binary_ep.len() > index + path_size {
            return Err(EndpointReadBytesError::TooLong());
        }
        let path = if path_size > 0 {
            Some(String::from_utf8(
                binary_ep[index..index + path_size].to_vec(),
            )?)
        } else {
            None
        };
        Ok(EndpointV2 {
            api,
            api_version,
            network_features,
            api_features,
            ip_v4,
            ip_v6,
            host,
            port,
            path,
            status: 0,
            last_check: 0,
        })
    }
}*/

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
    /// parse from ut8 format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        _status: u32,
        _last_check: u64,
    ) -> Result<EndpointEnum, ParseEndpointError> {
        let raw_ep_elements: Vec<&str> = raw_endpoint.split(' ').collect();
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
                        } else {
                            if let Ok(ip_v4) = Ipv4Addr::from_str(raw_ep_elements[index + 1]) {
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint
pub enum EndpointEnum {
    /// Endpoint v1
    V1(EndpointEnumV1),
    /// Endpoint v2
    V2(EndpointV2),
}

impl ToString for EndpointEnum {
    fn to_string(&self) -> String {
        match *self {
            EndpointEnum::V1(ref ep) => ep.raw_endpoint.clone(),
            EndpointEnum::V2(ref _ep) => panic!("Endpoint version is not supported !"),
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
    /// Accessors providing node full identifier
    pub fn node_full_id(&self) -> Option<NodeFullId> {
        match self.node_uuid() {
            Some(node_id) => Some(NodeFullId(node_id, self.pubkey())),
            None => None,
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
    /// Parse Endpoint from raw format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        issuer: PubKey,
        status: u32,
        last_check: u64,
        endpoint_version: u16,
    ) -> Result<EndpointEnum, ParseEndpointError> {
        match endpoint_version {
            1 => match ENDPOINT_V1_REGEX.captures(raw_endpoint) {
                Some(caps) => {
                    let node_id = match caps.name("uuid") {
                        Some(caps_node_id) => {
                            match u32::from_str_radix(caps_node_id.as_str(), 16) {
                                Ok(node_id) => Some(NodeId(node_id)),
                                Err(_) => None,
                            }
                        }
                        None => None,
                    };
                    let hash_full_id = match node_id {
                        Some(node_id_) => Some(NodeFullId(node_id_, issuer).sha256()),
                        None => None,
                    };
                    Ok(EndpointEnum::V1(EndpointEnumV1 {
                        version: 1,
                        issuer,
                        api: NetworkEndpointApi(String::from(&caps["api"])),
                        node_id,
                        hash_full_id,
                        host: String::from(&caps["host"]),
                        port: caps["port"].parse().unwrap_or(80),
                        path: match caps.name("path") {
                            Some(m) => Some(m.as_str().to_string()),
                            None => None,
                        },
                        raw_endpoint: String::from(raw_endpoint),
                        status,
                        last_check,
                    }))
                }
                None => Err(ParseEndpointError::WrongV1Format()),
            },
            2 => EndpointV2::parse_from_raw(raw_endpoint, status, last_check),
            _ => Err(ParseEndpointError::VersionNotSupported()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tests::bincode::{deserialize, serialize};

    fn test_parse_and_read_endpoint(str_endpoint: &str, endpoint: EndpointV2) {
        assert_eq!(
            EndpointV2::parse_from_raw(str_endpoint, 0, 0),
            Ok(EndpointEnum::V2(endpoint.clone())),
        );
        let binary_endpoint = serialize(&endpoint).expect("Fail to serialize endpoint !");
        let endpoint2: EndpointV2 =
            deserialize(&binary_endpoint).expect("Fail to deserialize endpoint !");
        assert_eq!(endpoint, endpoint2,)
    }

    #[test]
    fn test_parse_and_read_endpoint_with_host() {
        let str_endpoint = "WS2P 2 1 TLS 3 DEF LOW ABF 443 g1.durs.ifee.fr ws2p";
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
        let str_endpoint = "WS2P 2 1 TLS 3 DEF LOW ABF 443 84.16.72.210 ws2p";
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
        let str_endpoint = "WS2P 2 1 TLS 3 DEF LOW ABF 443 [2001:41d0:8:c5aa::1] ws2p";
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
        let str_endpoint =
            "WS2P 2 1 TLS 3 DEF LOW ABF 443 5.135.188.170 [2001:41d0:8:c5aa::1] ws2p";
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
}

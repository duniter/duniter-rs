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
use super::{NodeFullId, NodeUUID};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use duniter_crypto::keys::PubKey;
use duniter_documents::Hash;
use std::io::Cursor;
use std::mem;
use std::net::{AddrParseError, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;
use std::str::FromStr;

/// Total size of all fixed size fields of an EndpointV11
pub static ENDPOINTV11_FIXED_SIZE: &'static usize = &9;
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
    /// WrongV10Format
    WrongV10Format(),
    /// WrongV11Format (human-readable explanation)
    WrongV11Format(String),
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
pub struct NetworkEndpointV10 {
    /// API version
    pub version: usize,
    /// API Name
    pub api: NetworkEndpointApi,
    /// Node unique identifier
    pub node_id: Option<NodeUUID>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Api know by Duniter
pub enum ApiKnownByDuniter {
    /// BASIC_MERKLED_API
    BMA(),
    /// WebSocket To Peer
    WS2P(),
    /// GraphQL Verification Api
    GVA(),
    /// Duniter Advanced Statistic Api
    DASA(),
}

impl ApiKnownByDuniter {
    /// Convert ApiKnownByDuniter is their 8-bit binary value
    pub fn into_u8(self) -> u8 {
        match self {
            ApiKnownByDuniter::BMA() => 0u8,
            ApiKnownByDuniter::WS2P() => 1u8,
            ApiKnownByDuniter::GVA() => 2u8,
            ApiKnownByDuniter::DASA() => 3u8,
        }
    }
}

impl ToString for ApiKnownByDuniter {
    fn to_string(&self) -> String {
        match *self {
            ApiKnownByDuniter::BMA() => String::from("BMA"),
            ApiKnownByDuniter::WS2P() => String::from("WS2P"),
            ApiKnownByDuniter::GVA() => String::from("GVA"),
            ApiKnownByDuniter::DASA() => String::from("DASA"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Identifies the API of an endpointV2
pub enum EndpointV11Api {
    /// Api name is an 8-bit binary value
    Bin(ApiKnownByDuniter),
    /// Api name is a string utf8
    Str(String),
}

impl FromStr for EndpointV11Api {
    type Err = ParseEndpointError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BMA" => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::BMA())),
            "WS2P" => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::WS2P())),
            "GVA" => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::GVA())),
            "DASA" => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::DASA())),
            _ => {
                if s.len() <= ::std::u8::MAX as usize {
                    Ok(EndpointV11Api::Str(String::from(s)))
                } else {
                    Err(ParseEndpointError::ApiNameTooLong())
                }
            }
        }
    }
}

impl ToString for EndpointV11Api {
    fn to_string(&self) -> String {
        match *self {
            EndpointV11Api::Bin(ref api_bin_name) => api_bin_name.to_string(),
            EndpointV11Api::Str(ref api_name) => api_name.clone(),
        }
    }
}

impl EndpointV11Api {
    /// Get size of api name field
    pub fn size(&self) -> u8 {
        match *self {
            EndpointV11Api::Bin(_) => 0u8,
            EndpointV11Api::Str(ref api_name) => api_name.len() as u8,
        }
    }
    /// Convert api name into bytes vector
    pub fn into_bytes(&self) -> Vec<u8> {
        match *self {
            EndpointV11Api::Bin(api_bin_name) => vec![api_bin_name.into_u8()],
            EndpointV11Api::Str(ref api_name) => api_name.as_bytes().to_vec(),
        }
    }
    /// Get api from bytes
    pub fn api_from_bytes(
        api_size: usize,
        api_datas: &[u8],
    ) -> Result<EndpointV11Api, EndpointReadBytesError> {
        if api_size > 0 {
            if api_datas.len() == api_size {
                Ok(EndpointV11Api::Str(String::from_utf8(api_datas.to_vec())?))
            } else {
                Err(EndpointReadBytesError::WrongApiDatasLen())
            }
        } else if api_datas.len() == 1 {
            match api_datas[0] {
                0u8 => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::BMA())),
                1u8 => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::WS2P())),
                2u8 => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::GVA())),
                3u8 => Ok(EndpointV11Api::Bin(ApiKnownByDuniter::DASA())),
                _ => Err(EndpointReadBytesError::UnknowApiName()),
            }
        } else {
            Err(EndpointReadBytesError::WrongApiDatasLen())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Network features
pub struct EndpointV11NetworkFeatures(pub Vec<u8>);

impl EndpointV11NetworkFeatures {
    /// Parse network features from utf8 string's array
    pub fn from_str_array(
        str_array: &[&str],
    ) -> Result<EndpointV11NetworkFeatures, ParseEndpointError> {
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
        Ok(EndpointV11NetworkFeatures(vec![network_features]))
    }
    /// Network features size
    pub fn size(&self) -> u8 {
        self.0.len() as u8
    }
    /// Convert Self into bytes
    pub fn into_bytes(&self) -> &[u8] {
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
pub struct EndpointV11 {
    /// API Name
    pub api: EndpointV11Api,
    /// API version
    pub api_version: u16,
    /// Network features
    pub network_features: EndpointV11NetworkFeatures,
    /// API features
    pub api_features: Vec<u8>,
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
    /// Accessibility status of this endpoint  (updated regularly)
    pub status: u32,
    /// Timestamp of the last connection attempt to this endpoint
    pub last_check: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Size informations of Endpoint v2
pub struct EndpointV11Size {
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

impl EndpointV11Size {
    /// Compute total size of endpoint in binary format
    pub fn total_size(self) -> usize {
        let mut total_size = self.api_size as usize
            + self.host_size as usize
            + self.path_size as usize
            + self.nf_size as usize
            + self.af_size as usize
            + ENDPOINTV11_FIXED_SIZE;
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

impl EndpointV11 {
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, supported_ip_v6: bool) -> Option<String> {
        let protocol = self.api.to_string();
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
    /// get size of endpoint for binary format
    pub fn compute_endpoint_size(&self) -> EndpointV11Size {
        EndpointV11Size {
            api_size: self.api.size(),
            host_size: if let Some(ref host) = self.host {
                host.len() as u8
            } else {
                0u8
            },
            path_size: if let Some(ref path) = self.path {
                path.len() as u8
            } else {
                0u8
            },
            nf_size: self.network_features.size(),
            ip_v4: self.network_features.ip_v4(),
            ip_v6: self.network_features.ip_v6(),
            af_size: self.api_features.len() as u8,
        }
    }
    /// Convert endpoint into bytes vector
    pub fn into_bytes(self) -> Vec<u8> {
        let endpoint_size = self.compute_endpoint_size();
        let mut binary_endpoint = Vec::with_capacity(endpoint_size.total_size());
        binary_endpoint.push(endpoint_size.api_size);
        binary_endpoint.push(endpoint_size.host_size);
        binary_endpoint.push(endpoint_size.path_size);
        binary_endpoint.append(&mut self.api.into_bytes());
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
        binary_endpoint.extend_from_slice(&self.network_features.into_bytes());
        binary_endpoint.push(endpoint_size.af_size);
        binary_endpoint.append(&mut self.api_features.clone());
        if let Some(ip_v4) = self.ip_v4 {
            binary_endpoint.extend_from_slice(&ip_v4.octets());
        }
        if let Some(ip_v6) = self.ip_v6 {
            binary_endpoint.extend_from_slice(&ip_v6.octets());
        }
        if let Some(host) = self.host {
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
        if let Some(path) = self.path {
            binary_endpoint.extend_from_slice(path.as_bytes());
        }
        binary_endpoint
    }
    /// Create endpoint from bytes vector
    pub fn from_bytes(binary_ep: &[u8]) -> Result<EndpointV11, EndpointReadBytesError> {
        if binary_ep.len() < *ENDPOINTV11_FIXED_SIZE {
            return Err(EndpointReadBytesError::TooShort());
        }
        let api_size = binary_ep[0] as usize;
        let host_size = binary_ep[1] as usize;
        let path_size = binary_ep[2] as usize;
        if binary_ep.len() < (*ENDPOINTV11_FIXED_SIZE + api_size + host_size + path_size) {
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
        let api = EndpointV11Api::api_from_bytes(api_size, api_datas)?;
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
            EndpointV11NetworkFeatures(binary_ep[index..index + nf_size].to_vec());
        index += nf_size;
        // read af_size
        let af_size = binary_ep[index] as usize;
        index += 1;
        if binary_ep.len() < index + af_size + 1 {
            return Err(EndpointReadBytesError::TooShort());
        }
        // read api_features
        let api_features = binary_ep[index..index + nf_size].to_vec();
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
        Ok(EndpointV11 {
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
    /// parse from ut8 format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        status: u32,
        last_check: u64,
    ) -> Result<NetworkEndpoint, ParseEndpointError> {
        let raw_ep_elements: Vec<&str> = raw_endpoint.split(' ').collect();
        if raw_ep_elements.len() >= 6 {
            let api = EndpointV11Api::from_str(raw_ep_elements[0])?;
            let api_version: u16 = raw_ep_elements[1].parse()?;
            let network_features_count: usize = raw_ep_elements[2].parse()?;
            if network_features_count > *MAX_NETWORK_FEATURES_COUNT {
                Err(ParseEndpointError::MaxNetworkFeatures())
            } else if raw_ep_elements.len() >= 6 + network_features_count {
                let network_features = EndpointV11NetworkFeatures::from_str_array(
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
                        return Err(ParseEndpointError::WrongV11Format(String::from(
                            "All api features must be declared !",
                        )));
                    }
                    for i in (4 + network_features_count)
                        ..(4 + network_features_count + api_features_count)
                    {
                        if let Ok(feature) = raw_ep_elements[i].parse::<usize>() {
                            if feature > *MAX_API_FEATURES_COUNT {
                                return Err(ParseEndpointError::TooHighApiFeature());
                            }
                            let byte_index = feature / 8;
                            let feature = (feature % 8) as u8;
                            api_features[byte_index] += feature.pow(2);
                        } else if let EndpointV11Api::Bin(know_api) = api {
                            if let ApiKnownByDuniter::WS2P() = know_api {
                                match raw_ep_elements[i] {
                                    "DEF" => api_features[0] += 1u8,
                                    "LOW" => api_features[0] += 2u8,
                                    "ABF" => api_features[0] += 4u8,
                                    _ => {
                                        return Err(ParseEndpointError::UnknowApiFeature(
                                            String::from(raw_ep_elements[i]),
                                        ))
                                    }
                                }
                            } else {
                                return Err(ParseEndpointError::UnknowApiFeature(String::from(
                                    raw_ep_elements[i],
                                )));
                            }
                        } else {
                            return Err(ParseEndpointError::UnknowApiFeature(String::from(
                                raw_ep_elements[i],
                            )));
                        }
                    }
                    let mut index = 4 + network_features_count + api_features_count;
                    let ip_v4 = if network_features.ip_v4() {
                        let ip = Ipv4Addr::from_str(raw_ep_elements[index])?;
                        index += 1;
                        Some(ip)
                    } else {
                        None
                    };
                    let ip_v6 = if network_features.ip_v6() {
                        let ip = Ipv6Addr::from_str(raw_ep_elements[index])?;
                        index += 1;
                        Some(ip)
                    } else {
                        None
                    };
                    let (host, port) = if let Ok(port) = raw_ep_elements[index].parse::<u16>() {
                        index += 1;
                        (None, Some(port))
                    } else if raw_ep_elements.len() > index {
                        index += 2;
                        if let Ok(port) = raw_ep_elements[index - 1].parse::<u16>() {
                            (Some(String::from(raw_ep_elements[index - 2])), Some(port))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    };
                    if port.is_none() {
                        Err(ParseEndpointError::WrongV11Format(String::from(
                            "Missing port or is not integer !",
                        )))
                    } else {
                        let port = port.unwrap();
                        let path = if raw_ep_elements.len() > index {
                            index += 1;
                            Some(String::from(raw_ep_elements[index - 1]))
                        } else {
                            None
                        };
                        if raw_ep_elements.len() > index {
                            Err(ParseEndpointError::WrongV11Format(String::from(
                                "Too many fields !",
                            )))
                        } else {
                            Ok(NetworkEndpoint::V11(EndpointV11 {
                                api,
                                api_version,
                                network_features,
                                api_features: api_features.to_vec(),
                                ip_v4,
                                ip_v6,
                                host,
                                port,
                                path,
                                status,
                                last_check,
                            }))
                        }
                    }
                }
            } else {
                Err(ParseEndpointError::WrongV11Format(String::from(
                    "All network features must be declared !",
                )))
            }
        } else {
            Err(ParseEndpointError::WrongV11Format(String::from(
                "An endpoint must contain at least 6 elements",
            )))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Endpoint
pub enum NetworkEndpoint {
    /// Endpoint v1
    V10(NetworkEndpointV10),
    /// Endpoint v2
    V11(EndpointV11),
}

impl ToString for NetworkEndpoint {
    fn to_string(&self) -> String {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.raw_endpoint.clone(),
            NetworkEndpoint::V11(ref _ep_v11) => panic!("Endpoint version is not supported !"),
        }
    }
}

impl NetworkEndpoint {
    /// Accessors providing API name
    pub fn api(&self) -> NetworkEndpointApi {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.api.clone(),
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing node unique identifier
    pub fn node_uuid(&self) -> Option<NodeUUID> {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.node_id,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing node public key
    pub fn pubkey(&self) -> PubKey {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.issuer,
            _ => panic!("Endpoint version is not supported !"),
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
            NetworkEndpoint::V10(ref ep) => ep.port,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing raw format
    pub fn raw(&self) -> String {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.raw_endpoint.clone(),
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing endpoint accessibility status
    pub fn status(&self) -> u32 {
        match *self {
            NetworkEndpoint::V10(ref ep) => ep.status,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Set status
    pub fn set_status(&mut self, new_status: u32) {
        match *self {
            NetworkEndpoint::V10(ref mut ep) => ep.status = new_status,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Set last_check
    pub fn set_last_check(&mut self, new_last_check: u64) {
        match *self {
            NetworkEndpoint::V10(ref mut ep) => ep.last_check = new_last_check,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool, supported_ip_v6: bool) -> Option<String> {
        match *self {
            NetworkEndpoint::V10(ref ep) => {
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
            NetworkEndpoint::V11(ref ep_v11) => ep_v11.get_url(get_protocol, supported_ip_v6),
        }
    }
    /// Parse Endpoint from raw format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        issuer: PubKey,
        status: u32,
        last_check: u64,
        endpoint_version: u16,
    ) -> Result<NetworkEndpoint, ParseEndpointError> {
        match endpoint_version {
            1 => match ENDPOINT_V1_REGEX.captures(raw_endpoint) {
                Some(caps) => {
                    let node_id = match caps.name("uuid") {
                        Some(caps_node_id) => {
                            match u32::from_str_radix(caps_node_id.as_str(), 16) {
                                Ok(node_id) => Some(NodeUUID(node_id)),
                                Err(_) => None,
                            }
                        }
                        None => None,
                    };
                    let hash_full_id = match node_id {
                        Some(node_id_) => Some(NodeFullId(node_id_, issuer).sha256()),
                        None => None,
                    };
                    Ok(NetworkEndpoint::V10(NetworkEndpointV10 {
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
                None => Err(ParseEndpointError::WrongV10Format()),
            },
            2 => EndpointV11::parse_from_raw(raw_endpoint, status, last_check),
            _ => Err(ParseEndpointError::VersionNotSupported()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_read_endpoint() {
        let str_endpoint = "WS2P 2 1 TLS 3 DEF LOW ABF g1.durs.ifee.fr 443 ws2p";
        let endpoint = EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![4u8]),
            api_features: vec![7u8],
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        };
        assert_eq!(
            EndpointV11::parse_from_raw(str_endpoint, 0, 0),
            Ok(NetworkEndpoint::V11(endpoint.clone())),
        );
        let binary_endpoint = endpoint.clone().into_bytes();
        assert_eq!(
            EndpointV11::from_bytes(&binary_endpoint)
                .expect("Fail to convert byte vector into endpoint !"),
            endpoint,
        )
    }

    #[test]
    fn test_endpoint_to_bytes() {
        let endpoint_v11 = EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![4u8]),
            api_features: vec![7u8],
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        };
        assert_eq!(
            endpoint_v11.into_bytes(),
            vec![
                0, 15, 4, 1, 0, 2, 1, 4, 1, 7, 103, 49, 46, 100, 117, 114, 115, 46, 105, 102, 101,
                101, 46, 102, 114, 1, 187, 119, 115, 50, 112,
            ],
        )
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4() {
        let str_endpoint = "WS2P 2 2 IP4 TLS 3 DEF LOW ABF 84.16.72.210 443 ws2p";
        let endpoint = EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![5u8]),
            api_features: vec![7u8],
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        };
        assert_eq!(
            EndpointV11::parse_from_raw(str_endpoint, 0, 0),
            Ok(NetworkEndpoint::V11(endpoint.clone())),
        );
        let binary_endpoint = endpoint.clone().into_bytes();
        assert_eq!(
            EndpointV11::from_bytes(&binary_endpoint)
                .expect("Fail to convert byte vector into endpoint !"),
            endpoint
        )
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv6() {
        let str_endpoint = "WS2P 2 2 IP6 TLS 3 DEF LOW ABF 2001:41d0:8:c5aa::1 443 ws2p";
        let endpoint = EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![6u8]),
            api_features: vec![7u8],
            ip_v4: None,
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        };
        assert_eq!(
            EndpointV11::parse_from_raw(str_endpoint, 0, 0),
            Ok(NetworkEndpoint::V11(endpoint.clone())),
        );
        let binary_endpoint = endpoint.clone().into_bytes();
        assert_eq!(
            EndpointV11::from_bytes(&binary_endpoint)
                .expect("Fail to convert byte vector into endpoint !"),
            endpoint
        )
    }

    #[test]
    fn test_parse_and_read_endpoint_with_ipv4_and_ip_v6() {
        let str_endpoint =
            "WS2P 2 3 IP4 IP6 TLS 3 DEF LOW ABF 5.135.188.170 2001:41d0:8:c5aa::1 443 ws2p";
        let endpoint = EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![7u8]),
            api_features: vec![7u8],
            ip_v4: Some(Ipv4Addr::from_str("5.135.188.170").unwrap()),
            ip_v6: Some(Ipv6Addr::from_str("2001:41d0:8:c5aa::1").unwrap()),
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        };
        assert_eq!(
            EndpointV11::parse_from_raw(str_endpoint, 0, 0),
            Ok(NetworkEndpoint::V11(endpoint.clone())),
        );
        let binary_endpoint = endpoint.clone().into_bytes();
        assert_eq!(
            EndpointV11::from_bytes(&binary_endpoint)
                .expect("Fail to convert byte vector into endpoint !"),
            endpoint
        )
    }
}

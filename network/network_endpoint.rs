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
extern crate serde_json;

use self::regex::Regex;
use super::{NodeFullId, NodeUUID};
use duniter_crypto::keys::ed25519;
use duniter_documents::Hash;

lazy_static! {
    #[derive(Debug)]
    /// Regex match all endpoint in V1 format (works for all api)
    pub static ref ENDPOINT_V1_REGEX: Regex = Regex::new(
        r"^(?P<api>[A-Z0-9_]+) (?P<version>[1-9][0-9]*)? ?(?P<uuid>[a-f0-9]{6,8})? ?(?P<host>[a-z_][a-z0-9-_.]*|[0-9.]+|[0-9a-f:]+) (?P<port>[0-9]+)(?: /?(?P<path>.+)?)? *$"
    ).unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Identifies the API of an endpoint
pub struct NetworkEndpointApi(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Endpoint v1
pub struct NetworkEndpointV1 {
    /// API version
    pub version: usize,
    /// API Name
    pub api: NetworkEndpointApi,
    /// Node unique identifier
    pub node_id: Option<NodeUUID>,
    /// Public key of the node declaring this endpoint
    pub issuer: ed25519::PublicKey,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Endpoint
pub enum NetworkEndpoint {
    /// Endpoint v1
    V1(NetworkEndpointV1),
    /// Endpoint v2
    V2(),
}

impl ToString for NetworkEndpoint {
    fn to_string(&self) -> String {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.raw_endpoint.clone(),
            _ => panic!("Endpoint version is not supported !"),
        }
    }
}

impl NetworkEndpoint {
    /// Accessors providing API name
    pub fn api(&self) -> NetworkEndpointApi {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.api.clone(),
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing node unique identifier
    pub fn node_uuid(&self) -> Option<NodeUUID> {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.node_id,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing node public key
    pub fn pubkey(&self) -> ed25519::PublicKey {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.issuer,
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
            NetworkEndpoint::V1(ref ep) => ep.port,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing raw format
    pub fn raw(&self) -> String {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.raw_endpoint.clone(),
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Accessors providing endpoint accessibility status
    pub fn status(&self) -> u32 {
        match *self {
            NetworkEndpoint::V1(ref ep) => ep.status,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Set status
    pub fn set_status(&mut self, new_status: u32) {
        match *self {
            NetworkEndpoint::V1(ref mut ep) => ep.status = new_status,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Set last_check
    pub fn set_last_check(&mut self, new_last_check: u64) {
        match *self {
            NetworkEndpoint::V1(ref mut ep) => ep.last_check = new_last_check,
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Generate endpoint url
    pub fn get_url(&self, get_protocol: bool) -> String {
        match *self {
            NetworkEndpoint::V1(ref ep) => {
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
                    format!("{}{}://{}:{}/{}", protocol, tls, ep.host, ep.port, path)
                } else {
                    format!("{}:{}/{}", ep.host, ep.port, path)
                }
            }
            _ => panic!("Endpoint version is not supported !"),
        }
    }
    /// Parse Endpoint from rax format
    pub fn parse_from_raw(
        raw_endpoint: &str,
        issuer: ed25519::PublicKey,
        status: u32,
        last_check: u64,
    ) -> Option<NetworkEndpoint> {
        match ENDPOINT_V1_REGEX.captures(raw_endpoint) {
            Some(caps) => {
                let node_id = match caps.name("uuid") {
                    Some(caps_node_id) => match u32::from_str_radix(caps_node_id.as_str(), 16) {
                        Ok(node_id) => Some(NodeUUID(node_id)),
                        Err(_) => None,
                    },
                    None => None,
                };
                let hash_full_id = match node_id {
                    Some(node_id_) => Some(NodeFullId(node_id_, issuer).sha256()),
                    None => None,
                };
                Some(NetworkEndpoint::V1(NetworkEndpointV1 {
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
            None => None,
        }
    }
}

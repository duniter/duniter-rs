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

//! Defined all aspects of the inter-node network that concern all modules and are therefore independent of one implementation or another of this network layer.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate lazy_static;
//#[cfg(test)]
//#[macro_use]
//extern crate pretty_assertions;
#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate crypto;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_module;
extern crate dup_binarizer;
extern crate serde;
extern crate serde_json;

pub mod network_endpoint;
pub mod network_head;
pub mod network_peer;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use duniter_crypto::hashs::*;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::{
    BlockDocument, CertificationDocument, IdentityDocument, MembershipDocument, RevocationDocument,
    TransactionDocument,
};
use duniter_documents::blockchain::Document;
use duniter_documents::{BlockHash, BlockId, Blockstamp};
use duniter_module::*;
use network_head::NetworkHead;
use network_peer::PeerCard;
use std::fmt::{Debug, Display, Error, Formatter};
use std::ops::Deref;
use std::sync::mpsc;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
/// ApiFeatures
pub struct ApiFeatures(pub Vec<u8>);

/// ApiModule
pub trait ApiModule<DC: DuniterConf, M: ModuleMessage>: DuniterModule<DC, M> {
    /// Parsing error
    type ParseErr;
    /// Parse raw api features
    fn parse_raw_api_features(str_features: &str) -> Result<ApiFeatures, Self::ParseErr>;
}

/// NetworkModule
pub trait NetworkModule<DC: DuniterConf, M: ModuleMessage>: ApiModule<DC, M> {
    /// Launch synchronisation
    fn sync(
        soft_meta_datas: &SoftwareMetaDatas<DC>,
        keys: RequiredKeysContent,
        module_conf: <Self as DuniterModule<DC, M>>::ModuleConf,
        main_sender: mpsc::Sender<RooterThreadMessage<M>>,
        sync_endpoint: SyncEndpoint,
    ) -> Result<(), ModuleInitError>;
}

#[derive(Debug, Clone)]
/// Synchronisation endpoint
pub struct SyncEndpoint {
    /// Domaine name or IP
    pub domain_or_ip: String,
    /// Port number
    pub port: u16,
    /// Optionnal path
    pub path: Option<String>,
    /// Use TLS
    pub tls: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Random identifier with which several Duniter nodes with the same network keypair can be differentiated
pub struct NodeId(pub u32);

impl Default for NodeId {
    fn default() -> NodeId {
        NodeId(0)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{:x}", self.0)
    }
}

impl<'a> From<&'a str> for NodeId {
    fn from(source: &'a str) -> NodeId {
        NodeId(u32::from_str_radix(source, 16).expect("Fail to parse NodeId"))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Complete identifier of a duniter node.
pub struct NodeFullId(pub NodeId, pub PubKey);

impl Default for NodeFullId {
    fn default() -> NodeFullId {
        NodeFullId(
            NodeId::default(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                    .unwrap(),
            ),
        )
    }
}

impl Display for NodeFullId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}-{}", self.0, self.1)
    }
}

impl NodeFullId {
    /// Compute sha256 hash
    pub fn sha256(&self) -> Hash {
        let mut sha256 = Sha256::new();
        sha256.input_str(&format!("{}", self));
        Hash::from_hex(&sha256.result_str()).unwrap()
    }
    /// To human string
    pub fn to_human_string(&self) -> String {
        let mut pubkey_string = self.1.to_string();
        pubkey_string.truncate(8);
        format!("{:8x}-{:8}", (self.0).0, pubkey_string)
    }
}

/// Trait to be implemented by the configuration object of the module managing the inter-node network.
pub trait NetworkConf: Debug + Copy + Clone + PartialEq {}

#[derive(Debug, Clone)]
/// Block v10 in network format (Some events require a blockchain access to reconstitute the corresponding document)
pub struct NetworkBlockV10 {
    /// Uncompleted block document
    pub uncompleted_block_doc: BlockDocument,
    /// revoked
    pub revoked: Vec<serde_json::Value>,
    /// certifications
    pub certifications: Vec<serde_json::Value>,
}

#[derive(Debug, Clone)]
/// Block in network format (Some events require a blockchain access to reconstitute the corresponding document)
pub enum NetworkBlock {
    /// Block V10
    V10(Box<NetworkBlockV10>),
    /// Block V11
    V11(),
}

impl NetworkBlock {
    /// Return uncompleted block document
    pub fn uncompleted_block_doc(&self) -> BlockDocument {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => {
                network_block_v10.deref().uncompleted_block_doc.clone()
            }
            _ => panic!("Block version not supported !"),
        }
    }
    /// Return blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => {
                network_block_v10.deref().uncompleted_block_doc.blockstamp()
            }
            _ => panic!("Block version not supported !"),
        }
    }
    /// Return previous blockstamp
    pub fn previous_blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkBlock::V10(ref network_block_v10) => Blockstamp {
                id: BlockId(network_block_v10.deref().uncompleted_block_doc.number.0 - 1),
                hash: BlockHash(
                    network_block_v10
                        .deref()
                        .uncompleted_block_doc
                        .previous_hash,
                ),
            },
            _ => panic!("Block version not supported !"),
        }
    }
}

#[derive(Debug, Clone)]
/// Network Document
pub enum NetworkDocument {
    /// Network Block
    Block(NetworkBlock),
    /// Identity Document
    Identity(Box<IdentityDocument>),
    /// Membership Document
    Membership(Box<MembershipDocument>),
    /// Certification Document
    Certification(Box<CertificationDocument>),
    /// Revocation Document
    Revocation(Box<RevocationDocument>),
    /// Transaction Document
    Transaction(Box<TransactionDocument>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned when the network module fails to determine the current network consensus
pub enum NetworkConsensusError {
    /// The network module does not have enough data to determine consensus
    InsufficientData(usize),
    /// The network module does not determine consensus, there is most likely a fork
    Fork(),
}

#[derive(Debug, Clone)]
/// Type containing a request addressed to the network module
pub enum NetworkRequest {
    /// Get a current block of a specific node
    GetCurrent(ModuleReqFullId, NodeFullId),
    //GetBlock(ModuleReqFullId, NodeFullId, u64),
    /// Get a blocks chunk from specified node
    GetBlocks(ModuleReqFullId, NodeFullId, u32, u32),
    /// Get pending wot documents from specified node
    GetRequirementsPending(ModuleReqFullId, NodeFullId, u32),
    /// Obtain the current network consensus
    GetConsensus(ModuleReqFullId),
    /// Getting the heads cache
    GetHeadsCache(ModuleReqFullId),
    /// Get a list of known endpoints
    GetEndpoints(ModuleReqFullId),
}

impl NetworkRequest {
    /// Get request full identitifier
    pub fn get_req_full_id(&self) -> ModuleReqFullId {
        match *self {
            NetworkRequest::GetCurrent(ref req_id, _)
            | NetworkRequest::GetBlocks(ref req_id, _, _, _)
            | NetworkRequest::GetRequirementsPending(ref req_id, _, _)
            | NetworkRequest::GetConsensus(ref req_id)
            | NetworkRequest::GetHeadsCache(ref req_id)
            | NetworkRequest::GetEndpoints(ref req_id) => req_id.clone(),
        }
    }
    /// Get request identitifier
    pub fn get_req_id(&self) -> ModuleReqId {
        self.get_req_full_id().1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned when the network module does not get a satisfying answer to a request
pub enum NetworkRequestError {
    /// Receiving an invalid format response
    WrongFormat(),
    /// Unknow error
    UnknowError(),
    /// No response received
    NoResponse(),
    /// Unable to reach the target node
    ReceiverUnreachable(),
}

#[derive(Debug, Clone)]
/// Type containing the response to a network request
pub enum NetworkResponse {
    /// CurrentBlock
    CurrentBlock(ModuleReqFullId, NodeFullId, Box<NetworkBlock>),
    /// Block
    Block(ModuleReqFullId, NodeFullId, Box<NetworkBlock>),
    /// Chunk
    Chunk(ModuleReqFullId, NodeFullId, Vec<Box<NetworkBlock>>),
    /// PendingDocuments
    PendingDocuments(ModuleReqFullId, Vec<NetworkDocument>),
    /// Consensus
    Consensus(ModuleReqFullId, Result<Blockstamp, NetworkConsensusError>),
    /// HeadsCache
    HeadsCache(ModuleReqFullId, Box<NetworkHead>),
}

impl NetworkResponse {
    /// Get request full identifier
    pub fn get_req_full_id(&self) -> ModuleReqFullId {
        match *self {
            NetworkResponse::CurrentBlock(ref req_id, _, _)
            | NetworkResponse::Block(ref req_id, _, _)
            | NetworkResponse::Chunk(ref req_id, _, _)
            | NetworkResponse::PendingDocuments(ref req_id, _)
            | NetworkResponse::Consensus(ref req_id, _)
            | NetworkResponse::HeadsCache(ref req_id, _) => req_id.clone(),
        }
    }
    /// Get request identifier
    pub fn get_req_id(&self) -> ModuleReqId {
        self.get_req_full_id().1
    }
}

#[derive(Debug, Clone)]
/// Type containing a network event, each time a network event occurs it's relayed to all modules
pub enum NetworkEvent {
    /// Receiving a response to a network request
    ReqResponse(Box<NetworkResponse>),
    /// A connection has changed state(`u32` is the new state, `Option<String>` est l'uid du noeud)
    ConnectionStateChange(NodeFullId, u32, Option<String>, String),
    /// Receiving Pending Documents
    ReceiveDocuments(Vec<NetworkDocument>),
    /// Receipt of peer cards
    ReceivePeers(Vec<PeerCard>),
    /// Receiving heads
    ReceiveHeads(Vec<NetworkHead>),
}

#[cfg(test)]
mod tests {

    use super::network_endpoint::*;
    use super::*;

    #[test]
    fn parse_endpoint() {
        let issuer = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx")
                .unwrap(),
        );
        let node_id = NodeId(u32::from_str_radix("c1c39a0a", 16).unwrap());
        let full_id = NodeFullId(node_id, issuer);
        assert_eq!(
            NetworkEndpoint::parse_from_raw("WS2P c1c39a0a i3.ifee.fr 80 /ws2p", issuer, 0, 0, 1),
            Ok(NetworkEndpoint::V10(NetworkEndpointV10 {
                version: 1,
                issuer,
                api: NetworkEndpointApi(String::from("WS2P")),
                node_id: Some(node_id),
                hash_full_id: Some(full_id.sha256()),
                host: String::from("i3.ifee.fr"),
                port: 80,
                path: Some(String::from("ws2p")),
                raw_endpoint: String::from("WS2P c1c39a0a i3.ifee.fr 80 /ws2p"),
                last_check: 0,
                status: 0,
            },))
        );
    }

    #[test]
    fn parse_endpoint2() {
        let issuer = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("5gJYnQp8v7bWwk7EWRoL8vCLof1r3y9c6VDdnGSM1GLv")
                .unwrap(),
        );
        let node_id = NodeId(u32::from_str_radix("cb06a19b", 16).unwrap());
        let full_id = NodeFullId(node_id, issuer);
        assert_eq!(
            NetworkEndpoint::parse_from_raw("WS2P cb06a19b g1.imirhil.fr 53012 /", issuer, 0, 0, 1),
            Ok(NetworkEndpoint::V10(NetworkEndpointV10 {
                version: 1,
                issuer,
                api: NetworkEndpointApi(String::from("WS2P")),
                node_id: Some(node_id),
                hash_full_id: Some(full_id.sha256()),
                host: String::from("g1.imirhil.fr"),
                port: 53012,
                path: None,
                raw_endpoint: String::from("WS2P cb06a19b g1.imirhil.fr 53012 /"),
                last_check: 0,
                status: 0,
            },))
        );
    }
}

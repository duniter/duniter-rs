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

extern crate dubp_documents;
extern crate duniter_module;
extern crate dup_crypto;
extern crate durs_network_documents;
extern crate serde;
extern crate serde_json;

use dubp_documents::v10::block::BlockDocument;
use dubp_documents::v10::certification::CertificationDocument;
use dubp_documents::v10::identity::IdentityDocument;
use dubp_documents::v10::membership::MembershipDocument;
use dubp_documents::v10::revocation::RevocationDocument;
use dubp_documents::v10::transaction::TransactionDocument;
use dubp_documents::Document;
use dubp_documents::{blockstamp::Blockstamp, BlockHash, BlockId};
use duniter_module::*;
use durs_network_documents::network_endpoint::ApiFeatures;
use durs_network_documents::network_head::NetworkHead;
use durs_network_documents::network_peer::PeerCard;
use durs_network_documents::*;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::mpsc;

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
        main_sender: mpsc::Sender<RouterThreadMessage<M>>,
        sync_params: SyncParams,
    ) -> Result<(), ModuleInitError>;
}

/// SyncParams
#[derive(Debug, Clone)]
pub struct SyncParams {
    /// Synchronisation endpoint
    pub sync_endpoint: SyncEndpoint,
    /// Cautious flag
    pub cautious: bool,
    /// VERIF_HASHS flag
    pub verif_hashs: bool,
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
    /// Block V1
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
pub enum BlockchainDocument {
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

#[derive(Debug, Copy, Clone)]
/// Type containing a request addressed to the network module
pub enum OldNetworkRequest {
    /// Get a current block of a specific node
    GetCurrent(ModuleReqFullId, NodeFullId),
    //GetBlock(NodeFullId, u64),
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

impl OldNetworkRequest {
    /// Get request full identitifier
    pub fn get_req_full_id(&self) -> ModuleReqFullId {
        match *self {
            OldNetworkRequest::GetCurrent(ref req_id, _)
            | OldNetworkRequest::GetBlocks(ref req_id, _, _, _)
            | OldNetworkRequest::GetRequirementsPending(ref req_id, _, _)
            | OldNetworkRequest::GetConsensus(ref req_id)
            | OldNetworkRequest::GetHeadsCache(ref req_id)
            | OldNetworkRequest::GetEndpoints(ref req_id) => *req_id,
        }
    }
    /// Get request identitifier
    pub fn get_req_id(&self) -> ModuleReqId {
        self.get_req_full_id().1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned when the network module does not get a satisfying answer to a request
pub enum OldNetworkRequestError {
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
    PendingDocuments(ModuleReqFullId, Vec<BlockchainDocument>),
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
            | NetworkResponse::HeadsCache(ref req_id, _) => *req_id,
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
    /// A connection has changed state(`u32` is the new state, `Option<String>` est l'uid du noeud)
    ConnectionStateChange(NodeFullId, u32, Option<String>, String),
    /// Receiving Pending Documents
    ReceiveDocuments(Vec<BlockchainDocument>),
    /// Receipt of peer cards
    ReceivePeers(Vec<PeerCard>),
    /// Receiving heads
    ReceiveHeads(Vec<NetworkHead>),
}

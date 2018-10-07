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

//! Defined the few global types used by all modules,
//! as well as the DuniterModule trait that all modules must implement.

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

extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_module;
extern crate duniter_network;
extern crate serde;
extern crate serde_json;

use duniter_crypto::hashs::Hash;
use duniter_crypto::keys::Sig;
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALRequest, DALResponse};
use duniter_documents::blockchain::BlockchainProtocol;
use duniter_documents::BlockId;
use duniter_module::*;
use duniter_network::{NetworkEvent, NetworkRequest, NetworkResponse};

#[derive(Debug, Clone)]
/// Message exchanged between Durs modules
pub struct DursMsg(pub DursMsgReceiver, pub DursMsgContent);

impl ModuleMessage for DursMsg {}

/// The recipient of a message
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DursMsgReceiver {
    /// Message for all modules
    All,
    /// Message for one specific module
    One(ModuleStaticName),
    /// Message for all modules who play a specific role
    Role(ModuleRole),
    /// Message for all modules that are subscribed to a specific type of event
    Event(ModuleEvent),
}

#[derive(Debug, Clone)]
/// Content of message exchanged between Durs modules
pub enum DursMsgContent {
    /// Request
    Request(DursReq),
    /// Brut text message
    Text(String),
    /// Brut binary message
    Binary(Vec<u8>),
    /// New configuration of a module to save
    SaveNewModuleConf(ModuleName, serde_json::Value),
    /// Response of DALRequest
    DALResponse(Box<DALResponse>),
    /// Blockchain event
    DALEvent(DALEvent),
    /// Request to the network module
    NetworkRequest(NetworkRequest),
    /// Network event
    NetworkEvent(NetworkEvent),
    /// Response of NetworkRequest
    NetworkResponse(NetworkResponse),
    /// Pow module response
    ProverResponse(BlockId, Sig, u64),
    /// Client API event
    ReceiveDocsFromClient(Vec<BlockchainProtocol>),
    /// Stop signal
    Stop(),
}

#[derive(Debug, Clone)]
/// Durs modules requests
pub struct DursReq {
    /// Requester
    pub requester: ModuleStaticName,
    /// Request unique id
    pub id: ModuleReqId,
    /// Request content
    pub content: DursReqContent,
}

#[derive(Debug, Clone)]
/// Modules request content
pub enum DursReqContent {
    /// Blockchain datas request
    DALRequest(DALRequest),
    /// Request to the pow module
    ProverRequest(BlockId, Hash),
    /// Brut text request
    Text(String),
    /// Brut binary request
    Binary(Vec<u8>),
}

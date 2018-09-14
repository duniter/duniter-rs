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

use std::sync::mpsc;

use duniter_crypto::keys::Sig;
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALRequest, DALResponse};
use duniter_documents::blockchain::BlockchainProtocol;
use duniter_documents::{BlockId, Hash};
use duniter_module::{ModuleId, ModuleMessage};
use duniter_network::{NetworkEvent, NetworkRequest};

#[derive(Debug, Clone)]
/// Message exchanged between Duniter-rs modules
pub enum DuniterMessage {
    /// Brut text message
    Text(String),
    /// Brut binary message
    Binary(Vec<u8>),
    /// New configuration of a module to save
    SaveNewModuleConf(ModuleId, serde_json::Value),
    /// Subscriptions to the module feed
    Followers(Vec<mpsc::Sender<DuniterMessage>>),
    /// Blockchain datas request
    DALRequest(DALRequest),
    /// Response of DALRequest
    DALResponse(Box<DALResponse>),
    /// Blockchain event
    DALEvent(DALEvent),
    /// Request to the network module
    NetworkRequest(NetworkRequest),
    /// Network event
    NetworkEvent(NetworkEvent),
    /// Request to the pow module
    ProverRequest(BlockId, Hash),
    /// Pow module response
    ProverResponse(BlockId, Sig, u64),
    /// Client API event
    ReceiveDocsFromClient(Vec<BlockchainProtocol>),
    /// Stop signal
    Stop(),
}

impl ModuleMessage for DuniterMessage {}

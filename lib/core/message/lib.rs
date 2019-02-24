//  Copyright (C) 2018  The Durs Project Developers.
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

//! Define the format of the messages exchanged between the DURS modules.

#![allow(clippy::large_enum_variant)]
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

use duniter_module::*;
use durs_network_documents::network_endpoint::EndpointEnum;

/// Define modules events
pub mod events;

/// Define modules requests
pub mod requests;

/// Define requests responses
pub mod responses;

use crate::events::*;
use crate::requests::*;
use crate::responses::*;

/// Message exchanged between Durs modules
#[derive(Debug, Clone)]
pub enum DursMsg {
    /// Durs module event
    Event {
        /// Event type
        event_type: ModuleEvent,
        /// Event content
        event_content: DursEvent,
    },
    /// Durs modules requests
    Request {
        /// The requester
        req_from: ModuleStaticName,
        /// Recipient
        req_to: ModuleRole,
        /// Request id (Must be unique for a given requester)
        req_id: ModuleReqId,
        /// Request content
        req_content: DursReqContent,
    },
    /// Durs modules request response
    Response {
        /// The module that answers the request
        res_from: ModuleStaticName,
        /// The requester
        res_to: ModuleStaticName,
        /// Request id (Must be unique for a given requester)
        req_id: ModuleReqId,
        /// Response content
        res_content: DursResContent,
    },
    /// Stop signal
    Stop,
    /// New configuration of a module to save
    SaveNewModuleConf(ModuleStaticName, serde_json::Value),
    /// List of all endpoints declared by the modules
    ModulesEndpoints(Vec<EndpointEnum>),
}

impl ModuleMessage for DursMsg {}

/// Arbitrary datas
#[derive(Debug, Clone)]
pub enum ArbitraryDatas {
    /// Arbitrary text message
    Text(String),
    /// Arbitrary json message
    Json(serde_json::Value),
    /// Arbitrary binary message
    Binary(Vec<u8>),
}

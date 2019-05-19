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

//! WebSocketToPeer V2+ API Protocol.

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

pub mod connection_state;
pub mod constants;
pub mod controller;
pub mod orchestrator;
pub mod websocket;

use dup_crypto::keys::{KeyPair, KeyPairEnum};
use durs_network_documents::{NodeFullId, NodeId};
use durs_ws2p_messages::v2::api_features::WS2PFeatures;

/// Store self WS2P properties
#[derive(Debug, Clone, PartialEq)]
pub struct MySelfWs2pNode {
    /// Local node id
    pub my_node_id: NodeId,
    /// Local network keypair
    pub my_key_pair: KeyPairEnum,
    /// Local node WWS2PFeatures
    pub my_features: WS2PFeatures,
}

impl MySelfWs2pNode {
    /// Get self node full id
    pub fn get_full_id(&self) -> NodeFullId {
        NodeFullId(self.my_node_id, self.my_key_pair.public_key())
    }
}

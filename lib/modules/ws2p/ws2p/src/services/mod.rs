//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
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

//! WS2P Services

use dup_crypto::keys::KeyPairEnum;
use durs_network_documents::*;
use durs_ws2p_messages::v2::api_features::WS2PFeatures;

pub mod outgoing;

/// Websocket Error
#[derive(Debug, Copy, Clone)]
pub enum WsError {
    /// Unknown error
    UnknownError,
}

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

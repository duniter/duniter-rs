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

//! WS2P Services

use controllers::*;
use duniter_network::documents::BlockchainDocument;
use dup_crypto::keys::KeyPairEnum;
use durs_network_documents::network_head::NetworkHead;
use durs_network_documents::network_peer::PeerCard;
use durs_network_documents::*;
use durs_ws2p_messages::v2::api_features::WS2PFeatures;
use std::sync::mpsc;

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

/// Message for the ws2p service
#[derive(Debug, Clone)]
pub enum Ws2pServiceSender {
    /// Controller sender
    ControllerSender(mpsc::Sender<Ws2pControllerOrder>),
    /// A new incoming connection has been established
    NewIncomingConnection(NodeFullId),
    /// A connection has changed status
    ChangeConnectionState(NodeFullId, WS2PConnectionState),
    /// A valid head has been received
    ReceiveValidHead(NetworkHead),
    /// A valid peer has been received
    ReceiveValidPeer(PeerCard),
    /// A valid blockchain document has been received
    ReceiveValidDocument(BlockchainDocument),
}

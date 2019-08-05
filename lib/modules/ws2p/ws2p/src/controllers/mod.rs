//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! WS2P connections controllers.

//use constants::*;
use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use ws::Sender;
//use dup_crypto::keys::*;
use durs_network_documents::network_peer::PeerCardV11;
use durs_ws2p_messages::*;
//use std::sync::mpsc;

pub mod handler;
pub mod incoming_connections;
pub mod outgoing_connections;

/// Order transmitted to the controller
#[derive(Debug, Clone)]
pub enum Ws2pControllerOrder {
    /// Give a message to be transmitted
    SendMsg(Box<WS2PMessage>),
    /// Close the connection
    Close,
}

/// Store a websocket sender
pub struct WsSender(pub Sender);

impl ::std::fmt::Debug for WsSender {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "WsSender {{ }}")
    }
}

#[derive(Debug, Clone)]
/// WS2P remote node datas
pub struct Ws2pRemoteNodeDatas {
    /// Remote challenge
    pub challenge: Hash,
    /// Remote peer card
    pub peer_card: Option<PeerCardV11>,
    /// Remote current blockstamp
    pub current_blockstamp: Option<Blockstamp>,
}

#[derive(Debug, Copy, Clone)]
/// WS2P remote node request
pub enum Ws2pRemoteNodeReq {
    /// Sync
    Sync,
    /// Ask chunk
    AskChunk(Blockstamp),
}

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

//! WS2P connections controllers.

extern crate ws;

//use constants::*;
use self::ws::Sender;
use duniter_documents::Blockstamp;
use dup_crypto::hashs::Hash;
//use dup_crypto::keys::*;
use durs_network_documents::network_peer::PeerCardV11;
use durs_network_documents::*;
use durs_ws2p_messages::v2::api_features::WS2PFeatures;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use durs_ws2p_messages::*;
//use std::sync::mpsc;
use std::time::SystemTime;

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

#[derive(Debug, Copy, Clone, PartialEq)]
/// WS2P connection state
pub enum WS2PConnectionState {
    /// Never try to establish this connection
    NeverTry,
    /// Try to open websocket
    TryToOpenWS,
    /// Websocket error
    WSError,
    /// Try to send connect message
    TryToSendConnectMess,
    /// Endpoint unreachable
    Unreachable,
    /// Waiting connect message
    WaitingConnectMess,
    /// No response
    NoResponse,
    /// Negociation timeout
    NegociationTimeout,
    /// Receive valid connect message
    ConnectMessOk,
    /// Receive valid OK message but wait ACK message
    OkMessOkWaitingAckMess,
    /// Receive valid ACK message
    AckMessOk,
    /// Connection denial (maybe due to many different reasons : receive wrong message, wrong format, wrong signature, etc)
    Denial,
    /// Connection closed
    Close,
    /// Connection succesfully established
    Established,
}

#[derive(Debug, Clone)]
/// WS2P connection meta datas
pub struct Ws2pConnectionDatas {
    /// connect type
    pub connect_type: WS2Pv2ConnectType,
    /// Remote connect type
    pub remote_connect_type: Option<WS2Pv2ConnectType>,
    /// Connection state
    pub state: WS2PConnectionState,
    /// Connection features
    pub features: Option<WS2PFeatures>,
    /// Local challenge
    pub challenge: Hash,
    /// Remote node full id
    pub remote_full_id: Option<NodeFullId>,
    /// Remote node datas
    pub remote_datas: Option<Ws2pRemoteNodeDatas>,
    /// Timestamp of last received message
    pub last_mess_time: SystemTime,
    /// Indicator required for the anti-spam mechanism
    pub spam_interval: bool,
    /// Indicator required for the anti-spam mechanism
    pub spam_counter: usize,
}

impl Ws2pConnectionDatas {
    /// Instanciate new Ws2pConnectionDatas
    pub fn new(connect_type: WS2Pv2ConnectType) -> Self {
        Ws2pConnectionDatas {
            connect_type,
            remote_connect_type: None,
            state: WS2PConnectionState::TryToOpenWS,
            features: None,
            challenge: Hash::random(),
            remote_full_id: None,
            remote_datas: None,
            last_mess_time: SystemTime::now(),
            spam_interval: false,
            spam_counter: 0,
        }
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

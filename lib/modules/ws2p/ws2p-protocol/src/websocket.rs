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

//! WebSocketToPeer V2+ API Protocol.
//! Define websocket message, event and action

use std::net::SocketAddr;

#[derive(Clone, Debug)]
/// Websocket message
pub enum WebsocketMessage {
    /// Bnary message
    Bin(Vec<u8>),
    /// String message
    Str(String),
}

#[derive(Clone, Debug)]
/// Websocket incoming event
pub enum WebsocketIncomingEvent {
    /// Connection opening
    OnOpen {
        /// Remote addr
        remote_addr: Option<SocketAddr>,
    },
    /// Receive message
    OnMessage {
        /// Message content
        msg: WebsocketMessage,
    },
    /// Connection closed
    OnClose {
        /// Close code
        close_code: u16,
        /// Close reason
        reason: Option<String>,
    },
}

#[derive(Clone, Debug)]
/// Websocket action
pub enum WebsocketAction {
    /// Connect to websocket url
    ConnectTo {
        /// Websocket url
        url: String,
    },
    /// Send message in websocket
    SendMessage {
        /// message content
        msg: WebsocketMessage,
    },
    /// Close connection
    CloseConnection {
        /// Give a reason for the remote
        reason: Option<String>,
    },
}

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

//! Sub module that define WS2P connection state.

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// WS2P connection state
pub enum WS2PConnectionState {
    /// Never try to establish this connection
    NeverTry,
    /// Try to open websocket
    TryToOpenWS,
    /// Websocket error
    WSError,
    /// Try to send connect message
    TryToSendConnectMsg,
    /// Endpoint unreachable
    Unreachable,
    /// Waiting connect message
    WaitingConnectMsg,
    /// No response
    NoResponse,
    /// Negociation timeout
    NegociationTimeout,
    /// Receive valid connect message
    ConnectMessOk,
    /// Receive valid OK message but wait ACK message
    OkMsgOkWaitingAckMsg,
    /// Receive valid ACK message
    AckMsgOk,
    /// Receive valid SECRET_FLAGS message but wait ACK message
    SecretFlagsOkWaitingAckMsg,
    /// Receive valid SECRET_FLAGS message
    SecretFlagsOk,
    /// Connection denial (maybe due to many different reasons : receive wrong message, wrong format, wrong signature, etc)
    Denial,
    /// Connection closed
    Close,
    /// Connection successfully established
    Established,
}

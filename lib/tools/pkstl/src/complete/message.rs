//  Copyright (C) 2019  Eloïs SANCHEZ.
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

//! Manage complete Public Key Secure Transport Layer.
//! Sub-module define incoming messages format.

/// Incoming binary Message
#[derive(Debug, PartialEq)]
pub enum IncomingBinaryMessage {
    /// Connect message
    Connect {
        /// Your custom datas
        custom_datas: Option<Vec<u8>>,
        /// Peer public key of signature algorithm
        peer_sig_public_key: Vec<u8>,
    },
    /// Ack message
    Ack {
        /// Your custom datas
        custom_datas: Option<Vec<u8>>,
    },
    /// Message
    Message {
        /// Message datas (This is an option because it's possible to receive an empty message)
        datas: Option<Vec<u8>>,
    },
}

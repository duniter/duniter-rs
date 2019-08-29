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

//! Manage complete secure and decentralized transport layer with serialization/deserialization.

use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub mod deserializer;
pub mod serializer;

const HEADER_FORMAT_LEN: usize = 4;

/// Incoming Message
#[derive(Debug)]
pub enum IncomingMessage<M: Debug + DeserializeOwned> {
    /// Connect message
    Connect {
        /// Your custom datas
        custom_datas: Option<M>,
        /// Peer public key of signature algorithm
        peer_sig_public_key: Vec<u8>,
    },
    /// Ack message
    Ack {
        /// Your custom datas
        custom_datas: Option<M>,
    },
    /// Message
    Message {
        /// Message datas (This is an option because it's possible to receive an empty message)
        datas: Option<M>,
    },
}

#[derive(Debug)]
pub enum SerdeError {
    #[cfg(feature = "bin")]
    /// Bincode error
    BincodeError(String),
    #[cfg(feature = "cbor")]
    /// Cbor error
    CborError(serde_cbor::error::Error),
    #[cfg(feature = "json")]
    /// Json error
    JsonError(serde_json::Error),
    /// For the "raw binary" format, use the functions suffixed by _bin
    UseSuffixedBinFunctions,
    /// Not copyable error for linter
    _IoError(std::io::Error),
}

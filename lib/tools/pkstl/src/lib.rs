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

//! Public Key Secure Transport Layer.

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

mod agreement;
#[cfg(feature = "zip-sign")]
mod complete;
mod config;
mod constants;
mod digest;
mod encryption;
mod errors;
#[cfg(feature = "ser")]
mod format;
mod message;
mod minimal;
mod reader;
mod seeds;
mod signature;
mod status;

pub use agreement::EphemeralPublicKey;
pub use config::SecureLayerConfig;
pub use encryption::EncryptAlgo;
pub use errors::Error;
pub use message::{EncapsuledMessage, Message};
pub use minimal::MinimalSecureLayer;
pub use seeds::Seed32;
pub use signature::{SIG_ALGO_ED25519, SIG_ALGO_ED25519_ARRAY};

#[cfg(feature = "ser")]
pub use complete::IncomingMessage;
#[cfg(feature = "ser")]
pub use format::MessageFormat;

#[cfg(feature = "zip-sign")]
pub use complete::message::IncomingBinaryMessage;
#[cfg(feature = "zip-sign")]
pub use complete::SecureLayer;

/// PKSTL Result
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalNegoThread {
    Created,
    ConnectMsgSent,
    ValidAckMsgReceived,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RemoteNegoThread {
    WaitConnectMsg,
    ValidConnectMsgReceived,
    AckMsgSent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MsgType {
    Connect,
    Ack,
    UserMsg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Create(MsgType),
    Receive(MsgType),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionSideEffects {
    PushUserMsgIntoTmpStack,
}

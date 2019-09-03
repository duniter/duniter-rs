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

//! Define PKSTL deserializer.

use super::SerdeError;
use super::{IncomingMessage, HEADER_FORMAT_LEN};
use crate::format::MessageFormat;
use crate::{Error, IncomingBinaryMessage, Result, SecureLayer};
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use std::fmt::Debug;

pub(crate) fn read<M>(
    sl: &mut SecureLayer,
    incoming_datas: &[u8],
) -> Result<Vec<IncomingMessage<M>>>
where
    M: Debug + DeserializeOwned,
{
    let bin_msgs = sl.read_bin(incoming_datas)?;

    let mut msgs = Vec::new();

    for bin_msg in bin_msgs {
        match bin_msg {
            IncomingBinaryMessage::Connect {
                custom_datas,
                peer_sig_public_key,
            } => msgs.push(IncomingMessage::Connect {
                custom_datas: if let Some(custom_datas) = custom_datas {
                    Some(deserialize(&custom_datas)?)
                } else {
                    None
                },
                peer_sig_public_key,
            }),
            IncomingBinaryMessage::Ack { custom_datas } => msgs.push(IncomingMessage::Ack {
                custom_datas: if let Some(custom_datas) = custom_datas {
                    Some(deserialize(&custom_datas)?)
                } else {
                    None
                },
            }),
            IncomingBinaryMessage::Message { datas } => msgs.push(IncomingMessage::Message {
                datas: if let Some(datas) = datas {
                    Some(deserialize(&datas)?)
                } else {
                    None
                },
            }),
        };
    }
    Ok(msgs)
}

#[inline]
fn deserialize<M: Debug + DeserializeOwned>(binary_message: &[u8]) -> Result<M> {
    if binary_message.len() < HEADER_FORMAT_LEN {
        return Err(Error::RecvInvalidMsg(
            crate::errors::IncomingMsgErr::MessageTooShort,
        ));
    }

    // Read format
    let message_format = MessageFormat::try_from(&binary_message[..HEADER_FORMAT_LEN])?;

    deserialize_inner(&binary_message[HEADER_FORMAT_LEN..], message_format)
        .map_err(Error::SerdeError)
}

pub fn deserialize_inner<M>(
    binary_message: &[u8],
    message_format: MessageFormat,
) -> std::result::Result<M, SerdeError>
where
    M: Debug + DeserializeOwned,
{
    match message_format {
        MessageFormat::RawBinary => Err(SerdeError::UseSuffixedBinFunctions),
        #[cfg(feature = "bin")]
        MessageFormat::Bincode => Ok(bincode::deserialize::<M>(binary_message)
            .map_err(|e| SerdeError::BincodeError(format!("{}", e)))?),
        #[cfg(feature = "cbor")]
        MessageFormat::Cbor => {
            Ok(serde_cbor::from_slice::<M>(binary_message).map_err(SerdeError::CborError)?)
        }
        #[cfg(feature = "json")]
        MessageFormat::Utf8Json => {
            Ok(serde_json::from_slice::<M>(binary_message).map_err(SerdeError::JsonError)?)
        }
        _ => unimplemented!(),
    }
}

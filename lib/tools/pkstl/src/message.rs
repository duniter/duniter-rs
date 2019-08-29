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

//! Manage PKSTL messages.

use crate::constants::*;
use crate::digest::sha256;
use crate::{Error, Result};
use std::io::{BufWriter, Write};

const CONNECT_MSG_TYPE_HEADERS_SIZE: usize = 70;
const ACK_MSG_TYPE_HEADERS_SIZE: usize = 34;

const HEADERS_AND_FOOTERS_MAX_SIZE: usize = 136;

/// Message
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    /// Connect message
    Connect {
        /// Signature algorithm
        sig_algo: [u8; SIG_ALGO_LEN],
        /// Signature public key
        sig_pubkey: Vec<u8>,
        /// Custom datas
        custom_datas: Option<Vec<u8>>,
    },
    /// Ack Message
    Ack {
        /// Custom datas
        custom_datas: Option<Vec<u8>>,
    },
    /// Message
    Message {
        /// Custom datas
        custom_datas: Option<Vec<u8>>,
    },
}

/// Message that referencing datas
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageRef<'a> {
    /// Connect message
    Connect {
        /// Signature algorithm
        sig_algo: [u8; SIG_ALGO_LEN],
        /// Signature public key
        sig_pubkey: Vec<u8>,
        /// Custom datas
        custom_datas: Option<&'a [u8]>,
    },
    /// Ack Message
    Ack {
        /// Custom datas
        custom_datas: Option<&'a [u8]>,
    },
    /// Message
    Message {
        /// Custom datas
        custom_datas: Option<&'a [u8]>,
    },
}

/// Encapsuled message
#[derive(Debug, PartialEq)]
pub struct EncapsuledMessage {
    pub(crate) datas: Vec<u8>,
}

impl AsRef<[u8]> for EncapsuledMessage {
    fn as_ref(&self) -> &[u8] {
        &self.datas[..]
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum MsgTypeHeaders {
    Connect {
        peer_ephemeral_pk: [u8; EPK_SIZE],
        sig_algo: [u8; SIG_ALGO_LEN],
        sig_pubkey: Vec<u8>,
    },
    Ack {
        challenge: [u8; CHALLENGE_SIZE],
    },
    UserMsg,
}

impl MsgTypeHeaders {
    pub(crate) fn must_be_encrypted(&self) -> bool {
        if let MsgTypeHeaders::UserMsg = self {
            true
        } else {
            false
        }
    }
}

struct InnerPreparedMsg<'a> {
    bin_user_msg: Option<&'a [u8]>,
    type_msg_headers: Vec<u8>,
}

impl Message {
    pub(crate) fn from_bytes(msg_bytes: Vec<u8>, msg_type_headers: MsgTypeHeaders) -> Result<Self> {
        // Read custom datas
        let custom_datas = if !msg_bytes.is_empty() {
            Some(msg_bytes)
        } else {
            None
        };

        // Build message according to it's type
        match msg_type_headers {
            MsgTypeHeaders::UserMsg => Ok(Message::Message { custom_datas }),
            MsgTypeHeaders::Connect {
                sig_algo,
                sig_pubkey,
                ..
            } => Ok(Message::Connect {
                sig_algo,
                sig_pubkey,
                custom_datas,
            }),
            MsgTypeHeaders::Ack { .. } => Ok(Message::Ack { custom_datas }),
        }
    }
}

impl<'a> MessageRef<'a> {
    #[inline]
    fn prepare_message(
        &self,
        self_epk: &[u8],
        peer_epk: Option<&Vec<u8>>,
    ) -> Result<InnerPreparedMsg<'a>> {
        match self {
            Self::Connect {
                sig_algo,
                sig_pubkey,
                custom_datas,
            } => {
                // type message headers
                let mut type_msg_headers = Vec::with_capacity(CONNECT_MSG_TYPE_HEADERS_SIZE);
                type_msg_headers
                    .write(CONNECT_MSG_TYPE)
                    .map_err(Error::WriteError)?;
                type_msg_headers
                    .write(self_epk)
                    .map_err(Error::WriteError)?;
                type_msg_headers
                    .write(&sig_algo[..])
                    .map_err(Error::WriteError)?;
                type_msg_headers
                    .write(sig_pubkey)
                    .map_err(Error::WriteError)?;

                Ok(InnerPreparedMsg {
                    bin_user_msg: *custom_datas,
                    type_msg_headers,
                })
            }
            Self::Ack { custom_datas } => {
                // type message headers
                let mut type_msg_headers = Vec::with_capacity(ACK_MSG_TYPE_HEADERS_SIZE);
                type_msg_headers
                    .write(ACK_MSG_TYPE)
                    .map_err(Error::WriteError)?;
                // write challenge
                if let Some(peer_epk) = peer_epk {
                    Self::write_challenge(peer_epk, &mut type_msg_headers)?;
                } else {
                    panic!("Dev error: try to write ack message before known peer epk.");
                }

                Ok(InnerPreparedMsg {
                    bin_user_msg: *custom_datas,
                    type_msg_headers,
                })
            }
            Self::Message { custom_datas } => Ok(InnerPreparedMsg {
                bin_user_msg: *custom_datas,
                type_msg_headers: USER_MSG_TYPE.to_vec(),
            }),
        }
    }
    /// Convert message to bytes
    pub(crate) fn to_bytes(
        &self,
        self_epk: &[u8],
        peer_epk: Option<&Vec<u8>>,
    ) -> Result<EncapsuledMessage> {
        let InnerPreparedMsg {
            bin_user_msg,
            type_msg_headers,
        } = self.prepare_message(self_epk, peer_epk)?;

        let bin_user_msg_len = bin_user_msg.unwrap_or(&[]).len();

        // Create temporary write buffer for datas that will then be signed or hashed
        let mut bytes_will_signed_or_hashed = BufWriter::new(Vec::with_capacity(
            bin_user_msg_len + HEADERS_AND_FOOTERS_MAX_SIZE,
        ));

        // Write MAGIC_VALUE
        bytes_will_signed_or_hashed
            .write(&MAGIC_VALUE)
            .map_err(Error::WriteError)?;

        // Write VERSION
        bytes_will_signed_or_hashed
            .write(&CURRENT_VERSION)
            .map_err(Error::WriteError)?;

        // Write ENCAPSULED_MSG_SIZE
        let encapsuled_msg_size = type_msg_headers.len() + bin_user_msg.unwrap_or(&[]).len();
        bytes_will_signed_or_hashed
            .write(&(encapsuled_msg_size as u64).to_be_bytes())
            .map_err(Error::WriteError)?;

        // Write type message headers
        bytes_will_signed_or_hashed
            .write(&type_msg_headers)
            .map_err(Error::WriteError)?;

        // Write user message
        if let Some(bin_user_msg) = bin_user_msg {
            bytes_will_signed_or_hashed
                .write(bin_user_msg)
                .map_err(Error::WriteError)?;
        }

        // Flush bytes_will_signed buffer
        let bytes_will_signed_or_hashed = bytes_will_signed_or_hashed
            .into_inner()
            .map_err(|_| Error::BufferFlushError)?;

        // Return the bytes will signed
        Ok(EncapsuledMessage {
            datas: bytes_will_signed_or_hashed,
        })
    }
    #[inline]
    fn write_challenge<W: Write>(ephem_pk: &[u8], writer: &mut W) -> Result<()> {
        writer
            .write(sha256(ephem_pk).as_ref())
            .map_err(Error::WriteError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_encapsuled_message() {
        let encapsuled_msg = EncapsuledMessage {
            datas: vec![1, 2, 3],
        };

        assert_eq!(&[1, 2, 3], encapsuled_msg.as_ref());
    }

    #[test]
    fn test_connect_message_to_bytes() -> Result<()> {
        let fake_epk = &[0u8; 32];

        // Test connect message with custom datas
        let message = MessageRef::Connect {
            custom_datas: Some(&[5, 4, 4, 5]),
            sig_algo: [0u8; SIG_ALGO_LEN],
            sig_pubkey: vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15,
            ],
        };
        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 74, // ENCAPSULED_MSG_LEN
                    0, 1, // CONNECT_MSG_TYPE
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, // fake EPK (32 bytes)
                    0, 0, 0, 0, // SIG_ALGO
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
                    8, 9, 10, 11, 12, 13, 14, 15, // fake SIG_PK (32 bytes)
                    5, 4, 4, 5 // custom datas
                ],
            },
            message.to_bytes(fake_epk, None)?
        );

        // Test connect message without custom datas
        let message = MessageRef::Connect {
            custom_datas: None,
            sig_algo: [0u8; SIG_ALGO_LEN],
            sig_pubkey: vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
                10, 11, 12, 13, 14, 15,
            ],
        };

        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 70, // ENCAPSULED_MSG_LEN
                    0, 1, // CONNECT_MSG_TYPE
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, // fake EPK (32 bytes)
                    0, 0, 0, 0, // SIG_ALGO
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
                    8, 9, 10, 11, 12, 13, 14, 15, // fake SIG_PK (32 bytes)
                ],
            },
            message.to_bytes(fake_epk, None,)?
        );

        Ok(())
    }

    #[test]
    fn test_ack_message_to_bytes() -> Result<()> {
        let fake_epk = &[0u8; 32];

        // Test ack message with custom datas
        let message = MessageRef::Ack {
            custom_datas: Some(&[5, 4, 4, 5]),
        };
        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 38, // ENCAPSULED_MSG_LEN
                    0, 2, // ACK_MSG_TYPE
                    102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32,
                    8, 151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41,
                    37, // CHALLENGE (hash sha256)
                    5, 4, 4, 5 // custom datas
                ],
            },
            message.to_bytes(fake_epk, Some(&fake_epk.to_vec()),)?
        );

        // Test ack message without custom datas
        let message = MessageRef::Ack { custom_datas: None };
        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 34, // ENCAPSULED_MSG_LEN
                    0, 2, // ACK_MSG_TYPE
                    102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32,
                    8, 151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41,
                    37, // CHALLENGE (hash sha256)
                ],
            },
            message.to_bytes(fake_epk, Some(&fake_epk.to_vec()),)?
        );

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Dev error: try to write ack message before known peer epk.")]
    fn test_ack_message_to_bytes_before_recv_connect_msg() {
        let fake_epk = &[0u8; 32];

        // Test ack message without custom datas
        let message = MessageRef::Ack { custom_datas: None };
        let _ = message.to_bytes(fake_epk, None);
    }

    #[test]
    fn test_user_message_to_bytes() -> Result<()> {
        let fake_epk = &[0u8; 32];

        // Test user message
        let empty_user_message = MessageRef::Message {
            custom_datas: Some(&[5, 4, 4, 5]),
        };
        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 6, // ENCAPSULED_MSG_LEN
                    0, 0, // USER_MSG_TYPE
                    5, 4, 4, 5 // custom datas
                ],
            },
            empty_user_message.to_bytes(fake_epk, None)?
        );

        // Test empty user message
        let empty_user_message = MessageRef::Message { custom_datas: None };
        assert_eq!(
            EncapsuledMessage {
                datas: vec![
                    226, 194, 226, 210, // MAGIC_VALUE
                    0, 0, 0, 1, // VERSION
                    0, 0, 0, 0, 0, 0, 0, 2, // ENCAPSULED_MSG_LEN
                    0, 0, // USER_MSG_TYPE
                ],
            },
            empty_user_message.to_bytes(fake_epk, None)?
        );

        Ok(())
    }

    #[test]
    fn test_message_from_bytes() -> Result<()> {
        // Define message
        let mut msg_bytes = vec![
            3, 3, 3, 3, // user message
        ];

        // User message
        assert_eq!(
            Message::Message {
                custom_datas: Some(vec![3, 3, 3, 3]),
            },
            Message::from_bytes(msg_bytes.clone(), MsgTypeHeaders::UserMsg)?
        );

        // Ack message
        assert_eq!(
            Message::Ack {
                custom_datas: Some(vec![3, 3, 3, 3]),
            },
            Message::from_bytes(
                msg_bytes.clone(),
                MsgTypeHeaders::Ack {
                    challenge: [0u8; CHALLENGE_SIZE]
                }
            )?
        );

        // Connect message
        assert_eq!(
            Message::Connect {
                sig_pubkey: (0..31).collect(),
                sig_algo: [0u8; SIG_ALGO_LEN],
                custom_datas: Some(vec![3, 3, 3, 3]),
            },
            Message::from_bytes(
                msg_bytes.clone(),
                MsgTypeHeaders::Connect {
                    peer_ephemeral_pk: [0u8; EPK_SIZE],
                    sig_algo: [0u8; SIG_ALGO_LEN],
                    sig_pubkey: (0..31).collect(),
                }
            )?
        );

        // Truncate user message content, msut be not panic
        assert_eq!(
            Message::Message {
                custom_datas: Some(vec![3, 3])
            },
            Message::from_bytes(msg_bytes.drain(..2).collect(), MsgTypeHeaders::UserMsg)?,
        );

        // Empty message
        let empty_msg_bytes = vec![];
        assert_eq!(
            Message::Message { custom_datas: None },
            Message::from_bytes(empty_msg_bytes, MsgTypeHeaders::UserMsg)?
        );

        Ok(())
    }
}

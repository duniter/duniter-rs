//  Copyright (C) 2018  The Duniter Project Developers.
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

use super::ok::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use duniter_network::network_peer::{PeerCardReadBytesError, PeerCardV11};
use dup_binarizer::BinMessage;
use std::io::Cursor;
use std::mem;

/// WS2P v2 message payload metadata size
pub static WS2P_V2_MESSAGE_PAYLOAD_METADATA_SIZE: &'static usize = &8;

/// WS2Pv2MessagePayload
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WS2Pv2MessagePayload {
    //Connect,
    //Ack,
    //Flags,
    Ok(WS2Pv2OkMsg),
    //Ko,
    //Request
    Peers(Vec<PeerCardV11>),
    /*Headsv2(u16),
    Heads3(u16),
    Blocks(u16),
    PendingIdentities(u16),
    PendingMemberships(u16),
    PendingCerts(u16),
    PendingRevocations(u16),
    PendingTxs(u16),*/
}

/// WS2Pv2MessagePayload
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WS2Pv2MessageBinPayload {
    pub message_type: u16,
    pub elements_count: u16,
    pub payload_content: Vec<u8>,
}

/// WS2Pv2MessagePayloadReadBytesError
#[derive(Debug)]
pub enum WS2Pv2MessagePayloadReadBytesError {
    /// IoError
    IoError(::std::io::Error),
    /// TooShort
    TooShort(String),
    /// TooLong
    TooLong(),
    /// UnknowMsgType
    UnknowMsgType(),
    /// WrongElementsCount
    WrongElementsCount(),
    // WrongPayloadSize
    //WrongPayloadSize(),
    /// PeerCardReadBytesError
    PeerCardReadBytesError(PeerCardReadBytesError),
    /// ReadWS2Pv2OkMsgError
    ReadWS2Pv2OkMsgError(ReadWS2Pv2OkMsgError),
}

impl From<::std::io::Error> for WS2Pv2MessagePayloadReadBytesError {
    fn from(e: ::std::io::Error) -> Self {
        WS2Pv2MessagePayloadReadBytesError::IoError(e)
    }
}

impl From<ReadWS2Pv2OkMsgError> for WS2Pv2MessagePayloadReadBytesError {
    fn from(e: ReadWS2Pv2OkMsgError) -> Self {
        WS2Pv2MessagePayloadReadBytesError::ReadWS2Pv2OkMsgError(e)
    }
}

impl From<PeerCardReadBytesError> for WS2Pv2MessagePayloadReadBytesError {
    fn from(e: PeerCardReadBytesError) -> Self {
        WS2Pv2MessagePayloadReadBytesError::PeerCardReadBytesError(e)
    }
}

impl BinMessage for WS2Pv2MessagePayload {
    type ReadBytesError = WS2Pv2MessagePayloadReadBytesError;

    fn from_bytes(
        payload: &[u8],
    ) -> Result<WS2Pv2MessagePayload, WS2Pv2MessagePayloadReadBytesError> {
        // read payload_size
        let payload_size = if payload.len() >= 8 {
            let mut payload_size_bytes = Cursor::new(payload[4..8].to_vec());
            payload_size_bytes.read_u32::<BigEndian>()? as usize
        } else {
            return Err(WS2Pv2MessagePayloadReadBytesError::TooShort(String::from(
                "payload_size",
            )));
        };
        // Check size of bytes vector
        if payload.len() > 8 + payload_size {
            return Err(WS2Pv2MessagePayloadReadBytesError::TooLong());
        } else if payload.len() < 8 + payload_size {
            return Err(WS2Pv2MessagePayloadReadBytesError::TooShort(String::from(
                "payload_content",
            )));
        }
        // Read message_type
        let mut message_type_bytes = Cursor::new(payload[0..2].to_vec());
        let message_type = message_type_bytes.read_u16::<BigEndian>()?;
        // Read elements_count
        let mut elements_count_bytes = Cursor::new(payload[2..4].to_vec());
        let elements_count = elements_count_bytes.read_u16::<BigEndian>()? as usize;
        // Read payload_content
        match message_type {
            0x0003 => {
                if elements_count == 0 {
                    Ok(WS2Pv2MessagePayload::Ok(WS2Pv2OkMsg::from_bytes(
                        &payload[8..],
                    )?))
                } else {
                    Err(WS2Pv2MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0100 => {
                let mut peers = Vec::with_capacity(elements_count);
                let peers_bytes = &payload[8..];
                let mut index = 0;
                for _ in 0..elements_count {
                    // Read peer_size
                    index += 2;
                    if peers_bytes.len() < index {
                        return Err(WS2Pv2MessagePayloadReadBytesError::TooShort(String::from(
                            "peer_size",
                        )));
                    }
                    let mut peer_size_bytes = Cursor::new(peers_bytes[index - 2..index].to_vec());
                    let peer_size = peer_size_bytes.read_u16::<BigEndian>()? as usize;
                    // Read
                    index += peer_size;
                    if peers_bytes.len() < index {
                        return Err(WS2Pv2MessagePayloadReadBytesError::TooShort(String::from(
                            "peer_content",
                        )));
                    }
                    let peer = PeerCardV11::from_bytes(&peers_bytes[index - peer_size..index])?;
                    // Add peer
                    peers.push(peer);
                }
                if peers_bytes.len() > index {
                    Err(WS2Pv2MessagePayloadReadBytesError::TooLong())
                } else {
                    Ok(WS2Pv2MessagePayload::Peers(peers))
                }
            }
            _ => Err(WS2Pv2MessagePayloadReadBytesError::UnknowMsgType()),
        }
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        let bin_payload_container = match *self {
            WS2Pv2MessagePayload::Ok(ref ok_msg) => {
                WS2Pv2MessageBinPayload {
                    message_type: 0x0003,
                    elements_count: 0,
                    payload_content: ok_msg.to_bytes_vector(),
                }
            }
            WS2Pv2MessagePayload::Peers(ref peers) => {
                // Get elements_count
                let elements_count = peers.len() as u16;
                // Binarize peers
                let mut bin_peers = vec![];
                for peer in peers.clone() {
                    bin_peers.push(peer.to_bytes_vector());
                }
                // Compute payload_content_size
                let payload_size = bin_peers.iter().map(|bin_peer| bin_peer.len()).sum();
                let mut payload_content = Vec::with_capacity(payload_size);
                // Write payload_content
                for bin_peer in bin_peers {
                    payload_content.extend(bin_peer);
                }
                WS2Pv2MessageBinPayload {
                    message_type: 0x0100,
                    elements_count,
                    payload_content,
                }
            }
            //_ => vec![],
        };
        // Create bin_payload buffer
        let mut bin_payload = Vec::with_capacity(
            bin_payload_container.payload_content.len() + *WS2P_V2_MESSAGE_PAYLOAD_METADATA_SIZE,
        );
        // Write message_type
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(bin_payload_container.message_type)
            .expect("Unable to write");
        bin_payload.extend_from_slice(&buffer);
        // Write elements_count
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(bin_payload_container.elements_count)
            .expect("Unable to write");
        bin_payload.extend_from_slice(&buffer);
        // Write payload_size
        let mut buffer = [0u8; mem::size_of::<u32>()];
        buffer
            .as_mut()
            .write_u32::<BigEndian>(bin_payload_container.payload_content.len() as u32)
            .expect("Unable to write");
        bin_payload.extend_from_slice(&buffer);
        // Write payload_content
        bin_payload.extend(bin_payload_container.payload_content);
        // Return bin_payload
        bin_payload
    }
}

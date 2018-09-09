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

use super::connect::WS2Pv2ConnectMsg;
use super::ok::WS2Pv2OkMsg;
use super::req_responses::WS2Pv2ReqRes;
use super::requests::WS2Pv2Request;
use super::secret_flags::WS2Pv2SecretFlagsMsg;
use duniter_crypto::hashs::Hash;
use duniter_documents::blockchain::v10::documents::{
    BlockDocument, CertificationDocument, IdentityDocument, MembershipDocument, RevocationDocument,
    TransactionDocument,
};
use duniter_network::network_head_v2::NetworkHeadV2;
use duniter_network::network_head_v3::NetworkHeadV3Container;
use duniter_network::network_peer::PeerCardV11;

/// WS2P v2 message payload metadata size
pub static WS2P_V2_MESSAGE_PAYLOAD_METADATA_SIZE: &'static usize = &8;

/// WS2Pv0MessagePayload
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WS2Pv0MessagePayload {
    /// CONNECT message
    Connect(Box<WS2Pv2ConnectMsg>),
    /// ACK message
    Ack(Hash),
    /// SECRET_FLAGS Message
    SecretFlags(WS2Pv2SecretFlagsMsg),
    /// OK Message
    Ok(WS2Pv2OkMsg),
    /// KO Message
    Ko(u16),
    /// REQUEST Message
    Request(WS2Pv2Request),
    /// REQUEST_RESPONSE Message
    ReqRes(WS2Pv2ReqRes),
    /// PEERS Message
    Peers(Vec<PeerCardV11>),
    /// HEADS_V2 Message
    Headsv2(Vec<NetworkHeadV2>),
    /// HEADS_V3 Message
    Heads3(Vec<NetworkHeadV3Container>),
    /// BLOCKS Message
    Blocks(Vec<BlockDocument>),
    /// PENDING_IDENTITIES Message
    PendingIdentities(Vec<IdentityDocument>),
    /// PENDING_MEMBERSHIPS Message
    PendingMemberships(Vec<MembershipDocument>),
    /// PENDING_CERTS Message
    PendingCerts(Vec<CertificationDocument>),
    /// PENDING_REVOCATIONS Message
    PendingRevocations(Vec<RevocationDocument>),
    /// PENDING_TXS Message
    PendingTxs(Vec<TransactionDocument>),
}

/*
impl BinMessage for WS2Pv0MessagePayload {
    type ReadBytesError = WS2Pv0MessagePayloadReadBytesError;

    fn from_bytes(
        payload: &[u8],
    ) -> Result<WS2Pv0MessagePayload, WS2Pv0MessagePayloadReadBytesError> {
        // read payload_size
        let payload_size = if payload.len() >= 8 {
            let mut payload_size_bytes = Cursor::new(payload[4..8].to_vec());
            payload_size_bytes.read_u32::<BigEndian>()? as usize
        } else {
            return Err(WS2Pv0MessagePayloadReadBytesError::TooShort(String::from(
                "payload_size",
            )));
        };
        // Check size of bytes vector
        if payload.len() > 8 + payload_size {
            return Err(WS2Pv0MessagePayloadReadBytesError::TooLong());
        } else if payload.len() < 8 + payload_size {
            return Err(WS2Pv0MessagePayloadReadBytesError::TooShort(String::from(
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
            0x0000 => {
                if elements_count == 0 {
                    Ok(WS2Pv0MessagePayload::Connect(Box::new(
                        WS2Pv2ConnectMsg::from_bytes(&payload[8..])?,
                    )))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0001 => {
                if elements_count == 0 {
                    let mut hash_bytes = [0u8; 32];
                    hash_bytes.copy_from_slice(&payload[8..40]);
                    Ok(WS2Pv0MessagePayload::Ack(Hash(hash_bytes)))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0002 => {
                if elements_count == 0 {
                    Ok(WS2Pv0MessagePayload::SecretFlags(
                        WS2Pv2SecretFlagsMsg::from_bytes(&payload[8..])?,
                    ))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0003 => {
                if elements_count == 0 {
                    Ok(WS2Pv0MessagePayload::Ok(WS2Pv2OkMsg::from_bytes(
                        &payload[8..],
                    )?))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0010 => {
                if elements_count == 0 {
                    Ok(WS2Pv0MessagePayload::Request(WS2Pv2Request::from_bytes(
                        &payload[8..],
                    )?))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
                }
            }
            0x0011 => {
                if elements_count == 0 {
                    Ok(WS2Pv0MessagePayload::ReqRes(WS2Pv2ReqRes::from_bytes(
                        &payload[8..],
                    )?))
                } else {
                    Err(WS2Pv0MessagePayloadReadBytesError::WrongElementsCount())
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
                        return Err(WS2Pv0MessagePayloadReadBytesError::TooShort(String::from(
                            "peer_size",
                        )));
                    }
                    let mut peer_size_bytes = Cursor::new(peers_bytes[index - 2..index].to_vec());
                    let peer_size = peer_size_bytes.read_u16::<BigEndian>()? as usize;
                    // Read
                    index += peer_size;
                    if peers_bytes.len() < index {
                        return Err(WS2Pv0MessagePayloadReadBytesError::TooShort(String::from(
                            "peer_content",
                        )));
                    }
                    let peer = PeerCardV11::from_bytes(&peers_bytes[index - peer_size..index])?;
                    // Add peer
                    peers.push(peer);
                }
                if peers_bytes.len() > index {
                    Err(WS2Pv0MessagePayloadReadBytesError::TooLong())
                } else {
                    Ok(WS2Pv0MessagePayload::Peers(peers))
                }
            }
            _ => Err(WS2Pv0MessagePayloadReadBytesError::UnknowMsgType()),
        }
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        let bin_payload_container = match *self {
            WS2Pv0MessagePayload::Connect(ref connect_msg) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0000,
                    elements_count: 0,
                    payload_content: connect_msg.to_bytes_vector(),
                }
            }
            WS2Pv0MessagePayload::Ack(ref hash) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0001,
                    elements_count: 0,
                    payload_content: hash.0.to_vec(),
                }
            }
            WS2Pv0MessagePayload::SecretFlags(ref secret_flags_msg) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0002,
                    elements_count: 0,
                    payload_content: secret_flags_msg.to_bytes_vector(),
                }
            }
            WS2Pv0MessagePayload::Ok(ref ok_msg) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0003,
                    elements_count: 0,
                    payload_content: ok_msg.to_bytes_vector(),
                }
            }
            WS2Pv0MessagePayload::Request(ref request_msg) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0010,
                    elements_count: 0,
                    payload_content: request_msg.to_bytes_vector(),
                }
            }
            WS2Pv0MessagePayload::ReqRes(ref req_res_msg) => {
                WS2Pv0MessageBinPayload {
                    message_type: 0x0011,
                    elements_count: 0,
                    payload_content: req_res_msg.to_bytes_vector(),
                }
            }
            WS2Pv0MessagePayload::Peers(ref peers) => {
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
                WS2Pv0MessageBinPayload {
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
}*/

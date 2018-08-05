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

//! Crate containing Duniter-rust core.

// WS2P v2 Connect Messages
//pub mod connect;
/// Message Payload container
mod payload_container;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use duniter_crypto::hashs::Hash;
use duniter_crypto::keys::*;
use duniter_documents::{CurrencyCodeError, CurrencyName};
use duniter_network::NodeId;
use dup_binarizer::*;
use messages::v2::payload_container::*;
use std::io::Cursor;
use std::mem;

/// WS2P v2 message metadata size
pub static WS2P_V2_MESSAGE_METADATA_SIZE: &'static usize = &144;

/// WS2Pv2Message
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WS2Pv2Message {
    pub currency_code: CurrencyName,
    pub ws2p_version: u16,
    pub issuer_node_id: NodeId,
    pub issuer_pubkey: PubKey,
    pub payload: WS2Pv2MessagePayload,
    pub message_hash: Option<Hash>,
    pub signature: Option<Sig>,
}

/// WS2Pv2MessageReadBytesError
#[derive(Debug)]
pub enum WS2Pv2MessageReadBytesError {
    /// IoError
    IoError(::std::io::Error),
    /// CurrencyCodeError
    CurrencyCodeError(CurrencyCodeError),
    /// ReadPubkeyBoxError
    ReadPubkeyBoxError(pubkey_box::ReadPubkeyBoxError),
    /// ReadSigBoxError
    ReadSigBoxError(sig_box::ReadSigBoxError),
    /// TooShort
    TooShort(String),
    /// TooLong
    TooLong(),
    /// too early version (don't support binary format)
    TooEarlyVersion(),
    /// Version not yet supported
    VersionNotYetSupported(),
    /// WS2Pv2MessagePayloadReadBytesError
    WS2Pv2MessagePayloadReadBytesError(WS2Pv2MessagePayloadReadBytesError),
}

impl From<::std::io::Error> for WS2Pv2MessageReadBytesError {
    fn from(e: ::std::io::Error) -> Self {
        WS2Pv2MessageReadBytesError::IoError(e)
    }
}

impl From<CurrencyCodeError> for WS2Pv2MessageReadBytesError {
    fn from(e: CurrencyCodeError) -> Self {
        WS2Pv2MessageReadBytesError::CurrencyCodeError(e)
    }
}

impl From<pubkey_box::ReadPubkeyBoxError> for WS2Pv2MessageReadBytesError {
    fn from(e: pubkey_box::ReadPubkeyBoxError) -> Self {
        WS2Pv2MessageReadBytesError::ReadPubkeyBoxError(e)
    }
}

impl From<sig_box::ReadSigBoxError> for WS2Pv2MessageReadBytesError {
    fn from(e: sig_box::ReadSigBoxError) -> Self {
        WS2Pv2MessageReadBytesError::ReadSigBoxError(e)
    }
}

impl From<WS2Pv2MessagePayloadReadBytesError> for WS2Pv2MessageReadBytesError {
    fn from(error: WS2Pv2MessagePayloadReadBytesError) -> Self {
        WS2Pv2MessageReadBytesError::WS2Pv2MessagePayloadReadBytesError(error)
    }
}

impl BinMessageSignable for WS2Pv2Message {
    fn issuer_pubkey(&self) -> PubKey {
        self.issuer_pubkey
    }
    fn hash(&self) -> Option<Hash> {
        self.message_hash
    }
    fn set_hash(&mut self, hash: Hash) {
        self.message_hash = Some(hash)
    }
    fn signature(&self) -> Option<Sig> {
        self.signature
    }
    fn set_signature(&mut self, signature: Sig) {
        self.signature = Some(signature)
    }
}

impl BinMessage for WS2Pv2Message {
    type ReadBytesError = WS2Pv2MessageReadBytesError;
    fn from_bytes(binary_msg: &[u8]) -> Result<WS2Pv2Message, WS2Pv2MessageReadBytesError> {
        let mut index = 0;
        // read currency_code
        let mut currency_code_bytes = Cursor::new(binary_msg[index..index + 2].to_vec());
        index += 2;
        let currency_code = CurrencyName::from_u16(currency_code_bytes.read_u16::<BigEndian>()?)?;
        // read ws2p_version
        let mut ws2p_version_bytes = Cursor::new(binary_msg[index..index + 2].to_vec());
        index += 2;
        let ws2p_version = ws2p_version_bytes.read_u16::<BigEndian>()?;
        match ws2p_version {
            2u16 => {
                // read issuer_node_id
                let mut node_id_bytes = Cursor::new(binary_msg[index..index + 4].to_vec());
                index += 4;
                let issuer_node_id = NodeId(node_id_bytes.read_u32::<BigEndian>()?);
                // read issuer_size
                let issuer_size = u16::read_u16_be(&binary_msg[index..index + 2])? as usize;
                index += 2;
                // read issuer_pubkey
                let (issuer_pubkey, key_algo) = if binary_msg.len() > index + issuer_size {
                    index += issuer_size;
                    pubkey_box::read_pubkey_box(&binary_msg[index - issuer_size..index])?
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from(
                        "issuer",
                    )));
                };
                // read payload_size
                let payload_size = if binary_msg.len() > index + 8 {
                    let mut payload_size_bytes =
                        Cursor::new(binary_msg[index + 4..index + 8].to_vec());
                    payload_size_bytes.read_u32::<BigEndian>()? as usize
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from(
                        "payload_size",
                    )));
                };
                // read payload
                let payload = if binary_msg.len() > index + payload_size + 8 {
                    index += payload_size + 8;
                    WS2Pv2MessagePayload::from_bytes(
                        &binary_msg[index - (payload_size + 8)..index],
                    )?
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from(
                        "payload",
                    )));
                };
                // read message_hash
                let message_hash = if binary_msg.len() >= index + 32 {
                    let mut hash_datas: [u8; 32] = [0u8; 32];
                    index += 32;
                    hash_datas.copy_from_slice(&binary_msg[index - 32..index]);
                    Some(Hash(hash_datas))
                } else if binary_msg.len() == index {
                    None
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from(
                        "message_hash",
                    )));
                };
                // read signature_size
                let signature_size = if binary_msg.len() > index + 2 {
                    index += 2;
                    u16::read_u16_be(&binary_msg[index - 2..index])? as usize
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from(
                        "signature_size",
                    )));
                };
                // read signature
                let signature = if binary_msg.len() > index + signature_size {
                    return Err(WS2Pv2MessageReadBytesError::TooLong());
                } else if binary_msg.len() == index + signature_size {
                    index += signature_size;
                    Some(sig_box::read_sig_box(
                        &binary_msg[index - signature_size..index],
                        key_algo,
                    )?)
                } else if binary_msg.len() > index {
                    return Err(WS2Pv2MessageReadBytesError::TooLong());
                } else if binary_msg.len() == index {
                    None
                } else {
                    return Err(WS2Pv2MessageReadBytesError::TooShort(String::from("end")));
                };
                Ok(WS2Pv2Message {
                    currency_code,
                    ws2p_version,
                    issuer_node_id,
                    issuer_pubkey,
                    payload,
                    message_hash,
                    signature,
                })
            }
            0u16 | 1u16 => Err(WS2Pv2MessageReadBytesError::TooEarlyVersion()),
            _ => Err(WS2Pv2MessageReadBytesError::VersionNotYetSupported()),
        }
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        // Binarize payload (message_type + elements_count + payload_size + payload)
        let bin_payload = self.payload.to_bytes_vector();
        let payload_size = bin_payload.len() - *WS2P_V2_MESSAGE_PAYLOAD_METADATA_SIZE;
        let msg_size = *WS2P_V2_MESSAGE_METADATA_SIZE + payload_size as usize;
        let mut bytes_vector = Vec::with_capacity(msg_size);
        // currency_code
        bytes_vector.extend_from_slice(
            &self
                .currency_code
                .to_bytes()
                .expect("Fatal Error : Try to binarize WS2Pv2 message with UnknowCurrencyName !"),
        );
        // ws2p_version
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(self.ws2p_version)
            .expect("Unable to write");
        bytes_vector.extend_from_slice(&buffer);
        // Write issuer_node_id
        let mut buffer = [0u8; mem::size_of::<u32>()];
        buffer
            .as_mut()
            .write_u32::<BigEndian>(self.issuer_node_id.0)
            .expect("Unable to write");
        bytes_vector.extend_from_slice(&buffer);
        // Write issuer_pubey
        pubkey_box::write_pubkey_box(&mut bytes_vector, self.issuer_pubkey)
            .expect("Fail to binarize peer.issuer !");
        // Write payload : message_type + elements_count + payload_size + payload_content
        bytes_vector.extend(bin_payload);
        // Write message_hash
        if let Some(message_hash) = self.message_hash {
            bytes_vector.extend(message_hash.to_bytes_vector());
        }
        // Write signature
        if let Some(signature) = self.signature {
            sig_box::write_sig_box(&mut bytes_vector, signature)
                .expect("Fail to binarize msg.sig !");
        }

        bytes_vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use duniter_crypto::keys::*;
    use duniter_documents::Blockstamp;
    use duniter_network::network_endpoint::*;
    use duniter_network::network_peer::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    fn create_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![4u8]),
            api_features: vec![7u8],
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }
    fn create_second_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![5u8]),
            api_features: vec![7u8],
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }

    fn create_peer_card_v11() -> PeerCardV11 {
        PeerCardV11 {
            currency_code: 1u16,
            issuer: PubKey::Ed25519(keypair1().pubkey),
            node_id: NodeId(0),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            ).unwrap(),
            endpoints: vec![create_endpoint_v11(), create_second_endpoint_v11()],
            sig: None,
        }
    }

    #[test]
    fn test_ws2p_message_ok() {
        let keypair1 = keypair1();
        let mut ws2p_message = WS2Pv2Message {
            currency_code: CurrencyName(String::from("g1")),
            ws2p_version: 2u16,
            issuer_node_id: NodeId(0),
            issuer_pubkey: PubKey::Ed25519(keypair1.public_key()),
            payload: WS2Pv2MessagePayload::Ok,
            message_hash: None,
            signature: None,
        };

        let sign_result = ws2p_message.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(bin_msg) = sign_result {
            // Test binarization
            assert_eq!(ws2p_message.to_bytes_vector(), bin_msg);
            // Test sign
            assert_eq!(ws2p_message.verify(), Ok(()));
            // Test debinarization
            let debinarization_result = WS2Pv2Message::from_bytes(&bin_msg);
            if let Ok(ws2p_message2) = debinarization_result {
                assert_eq!(ws2p_message, ws2p_message2);
            } else {
                panic!(
                    "Fail to debinarize ws2p_message : {:?}",
                    debinarization_result.err().unwrap()
                );
            }
        } else {
            panic!(
                "Fail to sign ws2p_message : {:?}",
                sign_result.err().unwrap()
            );
        }
    }

    #[test]
    fn test_ws2p_message_peers() {
        let keypair1 = keypair1();
        let mut peer = create_peer_card_v11();
        peer.sign(PrivKey::Ed25519(keypair1.private_key()))
            .expect("Fail to sign peer card !");
        let mut ws2p_message = WS2Pv2Message {
            currency_code: CurrencyName(String::from("g1")),
            ws2p_version: 2u16,
            issuer_node_id: NodeId(0),
            issuer_pubkey: PubKey::Ed25519(keypair1.public_key()),
            payload: WS2Pv2MessagePayload::Peers(vec![peer]),
            message_hash: None,
            signature: None,
        };

        let sign_result = ws2p_message.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(bin_msg) = sign_result {
            // Test binarization
            assert_eq!(ws2p_message.to_bytes_vector(), bin_msg);
            // Test sign
            assert_eq!(ws2p_message.verify(), Ok(()));
            // Test debinarization
            let debinarization_result = WS2Pv2Message::from_bytes(&bin_msg);
            if let Ok(ws2p_message2) = debinarization_result {
                assert_eq!(ws2p_message, ws2p_message2);
            } else {
                panic!(
                    "Fail to debinarize ws2p_message : {:?}",
                    debinarization_result.err().unwrap()
                );
            }
        } else {
            panic!(
                "Fail to sign ws2p_message : {:?}",
                sign_result.err().unwrap()
            );
        }
    }
}

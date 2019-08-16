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

//! Handles WebSocketToPeer API Messages.

#![allow(clippy::large_enum_variant)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

/*#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;*/

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

/// WS2Pv2 Messages
pub mod v2;

use crate::v2::WS2Pv2Message;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::bin_signable::BinSignable;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
/// WS2Pv2Message
pub enum WS2PMessage {
    /// Old version not used
    _V0,
    /// Old version not used
    _V1,
    /// Version 2
    V2(WS2Pv2Message),
}

/// Enumerate errors can happen when parsing and checking messages
#[derive(Debug)]
pub enum WS2PMessageError {
    /// Error at deserialization
    DeserError(bincode::Error),
    /// Invalid hash
    InvalidHash,
    /// Invalid signature
    SigError(SigError),
}

impl From<bincode::Error> for WS2PMessageError {
    fn from(e: bincode::Error) -> Self {
        WS2PMessageError::DeserError(e)
    }
}

impl WS2PMessage {
    /// Get message hash
    pub fn hash(&self) -> Option<Hash> {
        match *self {
            WS2PMessage::V2(ref msg_v2) => msg_v2.message_hash,
            WS2PMessage::_V0 | WS2PMessage::_V1 => {
                fatal_error!("Dev error: must not use WS2PMessage version < 2 in WS2Pv2+ !")
            }
        }
    }

    /// Parse and check bin message
    pub fn parse_and_check_bin_message(bin_msg: &[u8]) -> Result<WS2PMessage, WS2PMessageError> {
        let msg: WS2PMessage = bincode::deserialize(&bin_msg)?;
        let hash = msg.hash();
        //debug!("parse_and_check_bin_message: hash={:?}", hash);
        // Compute hash len
        let hash_len = 33;
        // Compute signature len
        let sig_len = if let Some(sig) = msg.signature() {
            match sig {
                Sig::Ed25519(_) => 69,
                Sig::Schnorr() => fatal_error!("Schnorr algo not yet implemented !"),
            }
        } else {
            1
        };

        if hash.is_none()
            || Hash::compute(&bin_msg[0..(bin_msg.len() - hash_len - sig_len)])
                == hash.expect("safe unwrap")
        {
            match msg.verify() {
                Ok(()) => Ok(msg),
                Err(e) => Err(WS2PMessageError::SigError(e)),
            }
        } else {
            Err(WS2PMessageError::InvalidHash)
        }
    }
}

impl<'de> BinSignable<'de> for WS2PMessage {
    #[inline]
    fn add_sig_to_bin_datas(&self, bin_datas: &mut Vec<u8>) {
        bin_datas.extend_from_slice(
            &bincode::serialize(&self.signature()).expect("Fail to binarize sig !"),
        );
    }
    #[inline]
    fn get_bin_without_sig(&self) -> Result<Vec<u8>, failure::Error> {
        let mut bin_msg = bincode::serialize(&self)?;
        let sig_size = bincode::serialized_size(&self.signature())?;
        let bin_msg_len = bin_msg.len();
        bin_msg.truncate(bin_msg_len - (sig_size as usize));
        Ok(bin_msg)
    }
    fn issuer_pubkey(&self) -> PubKey {
        match *self {
            WS2PMessage::V2(ref msg_v2) => msg_v2.issuer_pubkey,
            WS2PMessage::_V0 | WS2PMessage::_V1 => {
                fatal_error!("Dev error: must not use WS2PMessage version < 2 in WS2Pv2+ !")
            }
        }
    }
    fn signature(&self) -> Option<Sig> {
        match *self {
            WS2PMessage::V2(ref msg_v2) => msg_v2.signature,
            WS2PMessage::_V0 | WS2PMessage::_V1 => {
                fatal_error!("Dev error: must not use WS2PMessage version < 2 in WS2Pv2+ !")
            }
        }
    }
    fn set_signature(&mut self, signature: Sig) {
        match *self {
            WS2PMessage::V2(ref mut msg_v2) => msg_v2.signature = Some(signature),
            WS2PMessage::_V0 | WS2PMessage::_V1 => {
                fatal_error!("Dev error: must not use WS2PMessage version < 2 in WS2Pv2+ !")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v2::payload_container::WS2Pv2MessagePayload;
    use crate::v2::WS2Pv2Message;
    use bincode;
    use bincode::{deserialize, serialize};
    use dubp_common_doc::{BlockNumber, Blockstamp};
    use dubp_currency_params::CurrencyName;
    use dubp_documents::documents::certification::*;
    use dup_crypto::keys::bin_signable::BinSignable;
    use dup_crypto::keys::*;
    use durs_network_documents::network_endpoint::*;
    use durs_network_documents::network_peer::*;
    use durs_network_documents::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    pub fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    pub fn create_endpoint_v11() -> EndpointV2 {
        EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![1u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            domain: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        }
    }
    pub fn create_second_endpoint_v11() -> EndpointV2 {
        EndpointV2 {
            api: ApiName(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![1u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            domain: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        }
    }

    pub fn create_peer_card_v11() -> PeerCardV11 {
        PeerCardV11 {
            currency_name: CurrencyName(String::from("g1")),
            issuer: PubKey::Ed25519(keypair1().pubkey),
            node_id: NodeId(0),
            created_on: BlockNumber(50),
            endpoints: vec![create_endpoint_v11(), create_second_endpoint_v11()],
            endpoints_str: vec![],
            sig: None,
        }
    }

    pub fn test_ws2p_message(payload: WS2Pv2MessagePayload) {
        let keypair1 = keypair1();
        let mut ws2p_message = WS2Pv2Message {
            currency_name: CurrencyName(String::from("g1")),
            issuer_node_id: NodeId(0),
            issuer_pubkey: PubKey::Ed25519(keypair1.public_key()),
            payload,
            message_hash: None,
            signature: None,
        };

        let sign_result = ws2p_message.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(bin_msg) = sign_result {
            // Test binarization
            assert_eq!(
                serialize(&ws2p_message).expect("Fail to serialize WS2Pv2Message !"),
                bin_msg
            );
            // Test sign
            ws2p_message
                .verify()
                .expect("WS2Pv2Message : Invalid signature !");
            // Test debinarization
            let debinarization_result: Result<WS2Pv2Message, bincode::Error> =
                deserialize(&bin_msg);
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

    pub fn create_cert_doc() -> CompactCertificationDocumentV10 {
        let sig = Sig::Ed25519(ed25519::Signature::from_base64(
            "qfR6zqT1oJbqIsppOi64gC9yTtxb6g6XA9RYpulkq9ehMvqg2VYVigCbR0yVpqKFsnYiQTrnjgFuFRSJCJDfCw==",
        ).unwrap());

        let target = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let blockstamp = Blockstamp::from_string(
            "36-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B865",
        )
        .unwrap();

        CompactCertificationDocumentV10 {
            issuer: PubKey::Ed25519(
                ed25519::PublicKey::from_base58("4tNQ7d9pj2Da5wUVoW9mFn7JjuPoowF977au8DdhEjVR")
                    .unwrap(),
            ),
            target,
            block_number: blockstamp.id,
            signature: sig,
        }
    }
}

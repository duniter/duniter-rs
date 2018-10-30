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

//! Handles WebSocketToPeer API Messages.

#![cfg_attr(feature = "strict", deny(warnings))]
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

extern crate bincode;
extern crate byteorder;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_network;

/// WS2Pv2 Messages
pub mod v2;

use v2::WS2Pv0Message;

#[derive(Debug, Clone, Eq, PartialEq)]
/// WS2Pv0Message
pub enum WS2PMessage {
    /// Version 2
    V0(WS2Pv0Message),
}

#[cfg(test)]
mod tests {
    use bincode;
    use bincode::{deserialize, serialize};
    use duniter_crypto::keys::bin_signable::BinSignable;
    use duniter_crypto::keys::*;
    use duniter_documents::v10::certification::*;
    use duniter_documents::{Blockstamp, CurrencyName};
    use duniter_network::network_endpoint::*;
    use duniter_network::network_peer::*;
    use duniter_network::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use v2::payload_container::WS2Pv0MessagePayload;
    use v2::WS2Pv0Message;

    pub fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    pub fn create_endpoint_v11() -> EndpointEnum {
        EndpointEnum::V2(EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        })
    }
    pub fn create_second_endpoint_v11() -> EndpointEnum {
        EndpointEnum::V2(EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![5u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        })
    }

    pub fn create_peer_card_v11() -> PeerCardV11 {
        PeerCardV11 {
            currency_name: CurrencyName(String::from("g1")),
            issuer: PubKey::Ed25519(keypair1().pubkey),
            node_id: NodeId(0),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            )
            .unwrap(),
            endpoints: vec![create_endpoint_v11(), create_second_endpoint_v11()],
            sig: None,
        }
    }

    pub fn test_ws2p_message(payload: WS2Pv0MessagePayload) {
        let keypair1 = keypair1();
        let mut ws2p_message = WS2Pv0Message {
            currency_code: CurrencyName(String::from("g1")),
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
                serialize(&ws2p_message).expect("Fail to serialize WS2Pv0Message !"),
                bin_msg
            );
            // Test sign
            ws2p_message
                .verify()
                .expect("WS2Pv0Message : Invalid signature !");
            // Test debinarization
            let debinarization_result: Result<WS2Pv0Message, bincode::Error> =
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

    pub fn create_cert_doc() -> CompactCertificationDocument {
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

        CompactCertificationDocument {
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

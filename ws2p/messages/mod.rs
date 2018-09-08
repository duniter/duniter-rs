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

/// WS2P v2 Messages
pub mod v2;

#[cfg(test)]
mod tests {
    use duniter_crypto::keys::*;
    use duniter_documents::{Blockstamp, CurrencyName};
    use duniter_network::network_endpoint::*;
    use duniter_network::network_peer::*;
    use duniter_network::*;
    use dup_binarizer::{BinMessage, BinMessageSignable};
    use messages::v2::payload_container::WS2Pv2MessagePayload;
    use messages::v2::WS2Pv2Message;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    pub fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }

    pub fn create_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }
    pub fn create_second_endpoint_v11() -> EndpointV11 {
        EndpointV11 {
            api: EndpointV11Api::Bin(ApiKnownByDuniter::WS2P()),
            api_version: 2,
            network_features: EndpointV11NetworkFeatures(vec![5u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
            status: 0,
            last_check: 0,
        }
    }

    pub fn create_peer_card_v11() -> PeerCardV11 {
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

    pub fn test_ws2p_message(payload: WS2Pv2MessagePayload) {
        let keypair1 = keypair1();
        let mut ws2p_message = WS2Pv2Message {
            currency_code: CurrencyName(String::from("g1")),
            ws2p_version: 2u16,
            issuer_node_id: NodeId(0),
            issuer_pubkey: PubKey::Ed25519(keypair1.public_key()),
            payload,
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

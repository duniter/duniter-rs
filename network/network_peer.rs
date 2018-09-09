//  Copyright (C) 2017  The Duniter Project Developers.
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

//! Module defining the format of network peer cards and how to handle them.

extern crate crypto;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_module;
extern crate serde;

use duniter_crypto::keys::*;
use duniter_documents::{Blockstamp, CurrencyName};
use dup_binarizer::*;
use network_endpoint::*;
use *;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card V10
pub struct PeerCardV10 {
    /// Peer card Blockstamp
    pub blockstamp: Blockstamp,
    /// Peer card issuer
    pub issuer: PubKey,
    /// Peer card endpoints list
    pub endpoints: Vec<EndpointEnum>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Peer card V11
pub struct PeerCardV11 {
    /// Currency name
    pub currency_name: CurrencyName,
    /// Peer card issuer
    pub issuer: PubKey,
    /// Issuer node id
    pub node_id: NodeId,
    /// Peer card Blockstamp
    pub blockstamp: Blockstamp,
    /// Peer card endpoints list
    pub endpoints: Vec<EndpointEnum>,
    /// Signature
    pub sig: Option<Sig>,
}

impl<'de> BinMessageSignable<'de> for PeerCardV11 {
    fn issuer_pubkey(&self) -> PubKey {
        self.issuer
    }
    fn signature(&self) -> Option<Sig> {
        self.sig
    }
    fn set_signature(&mut self, signature: Sig) {
        self.sig = Some(signature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card
pub enum PeerCard {
    /// Peer card V10
    V10(PeerCardV10),
    /// Peer card V11
    V11(PeerCardV11),
}

impl PeerCard {
    /// Get peer card version
    pub fn version(&self) -> u32 {
        match *self {
            PeerCard::V10(ref _peer_v10) => 10,
            PeerCard::V11(ref _peer_v11) => 11,
        }
    }
    /// Get peer card blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.blockstamp,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Get peer card issuer
    pub fn issuer(&self) -> PubKey {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.issuer,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Verify validity of peer card signature
    pub fn verify(&self) -> bool {
        false
    }
    /// Get peer card endpoint
    pub fn get_endpoints(&self) -> Vec<EndpointEnum> {
        Vec::with_capacity(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use tests::bincode::deserialize;

    fn keypair1() -> ed25519::KeyPair {
        ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        )
    }
    fn create_endpoint_v2() -> EndpointV2 {
        EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![4u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: None,
            ip_v6: None,
            host: Some(String::from("g1.durs.ifee.fr")),
            port: 443u16,
            path: Some(String::from("ws2p")),
        }
    }
    fn create_second_endpoint_v2() -> EndpointV2 {
        EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![5u8]),
            api_features: ApiFeatures(vec![7u8]),
            ip_v4: Some(Ipv4Addr::from_str("84.16.72.210").unwrap()),
            ip_v6: None,
            host: None,
            port: 443u16,
            path: Some(String::from("ws2p")),
        }
    }

    #[test]
    fn test_convert_peer_card_v11_into_bytes_vector() {
        let keypair1 = keypair1();
        let mut peer_card_v11 = PeerCardV11 {
            currency_name: CurrencyName(String::from("g1")),
            issuer: PubKey::Ed25519(keypair1.public_key()),
            node_id: NodeId(0),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            ).unwrap(),
            endpoints: vec![
                EndpointEnum::V2(create_endpoint_v2()),
                EndpointEnum::V2(create_second_endpoint_v2()),
            ],
            sig: None,
        };
        // Sign
        let sign_result = peer_card_v11.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(peer_card_v11_bytes) = sign_result {
            let deser_peer_card_v11: PeerCardV11 =
                deserialize(&peer_card_v11_bytes).expect("Fail to deserialize PeerCardV11 !");
            assert_eq!(peer_card_v11, deser_peer_card_v11,)
        } else {
            panic!("failt to sign peer card : {:?}", sign_result.err().unwrap())
        }
    }
}

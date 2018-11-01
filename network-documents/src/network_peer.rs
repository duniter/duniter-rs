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

extern crate duniter_documents;
extern crate dup_crypto;
extern crate serde;

use base58::ToBase58;
use duniter_documents::{blockstamp::Blockstamp, CurrencyName};
use dup_crypto::keys::bin_signable::BinSignable;
use dup_crypto::keys::*;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Peer card V11 for JSON Serializer
pub struct JsonPeerCardV11<'a> {
    /// PeerCard version
    pub version: usize,
    /// Currency Name
    pub currency_name: &'a str,
    /// Issuer node id
    pub node_id: NodeId,
    /// Issuer pubkey alogirithm
    pub algorithm: KeysAlgo,
    /// Issuer pubkey
    pub pubkey: String,
    /// Peer card creation blockstamp
    pub blockstamp: String,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Signature
    pub signature: Option<String>,
}

impl PeerCardV11 {
    /// Convert to JSON String
    pub fn to_json_peer(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string_pretty(&JsonPeerCardV11 {
            version: 11,
            currency_name: &self.currency_name.0,
            node_id: self.node_id,
            algorithm: self.issuer.algo(),
            pubkey: self.issuer.to_base58(),
            blockstamp: self.blockstamp.to_string(),
            endpoints: self.endpoints.iter().map(|ep| ep.to_string()).collect(),
            signature: if let Some(sig) = self.sig {
                Some(sig.to_base64())
            } else {
                None
            },
        })?)
    }
}

impl<'de> BinSignable<'de> for PeerCardV11 {
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
    use tests::keypair1;

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
    fn peer_card_v11_sign_and_verify() {
        let keypair1 = keypair1();
        let mut peer_card_v11 = PeerCardV11 {
            currency_name: CurrencyName(String::from("g1")),
            issuer: PubKey::Ed25519(keypair1.public_key()),
            node_id: NodeId(0),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            )
            .unwrap(),
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
        // Verify signature
        peer_card_v11
            .verify()
            .expect("Fail to verify PeerCardV11 !");
    }
}

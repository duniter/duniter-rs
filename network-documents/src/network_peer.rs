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

extern crate dubp_documents;
extern crate dup_crypto;
extern crate serde;

use base58::ToBase58;
use dubp_documents::{blockstamp::Blockstamp, CurrencyName};
use dubp_documents::{BlockHash, BlockId};
use dup_crypto::keys::text_signable::TextSignable;
use dup_crypto::keys::*;
use network_endpoint::*;
use pest::iterators::Pair;
use pest::Parser;
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
    /// Peer card binary endpoints
    pub endpoints: Vec<EndpointV2>,
    /// Peer card string endpoints
    pub endpoints_str: Vec<String>,
    /// Signature
    pub sig: Option<Sig>,
}

impl TextSignable for PeerCardV11 {
    fn as_signable_text(&self) -> String {
        format!(
            "11:{currency}:{node_id}:{pubkey}:{blockstamp}\n{endpoinds}\n{endpoints_str}{nl}",
            currency = self.currency_name.0,
            node_id = format!("{}", self.node_id),
            pubkey = self.issuer.to_base58(),
            blockstamp = self.blockstamp.to_string(),
            endpoinds = self
                .endpoints
                .iter()
                .map(EndpointV2::to_string)
                .collect::<Vec<String>>()
                .join("\n"),
            endpoints_str = self.endpoints_str.join("\n"),
            nl = if self.endpoints_str.is_empty() {
                ""
            } else {
                "\n"
            },
        )
    }
    fn issuer_pubkey(&self) -> PubKey {
        self.issuer
    }
    fn signature(&self) -> Option<Sig> {
        self.sig
    }
    fn set_signature(&mut self, signature: Sig) {
        self.sig = Some(signature);
    }
}

impl PeerCardV11 {
    /// parse from raw ascii format
    pub fn parse_from_raw(raw_peer: &str) -> Result<PeerCardV11, ParseError> {
        match NetworkDocsParser::parse(Rule::peer_v11, raw_peer) {
            Ok(mut peer_v11_pairs) => {
                Ok(PeerCardV11::from_pest_pair(peer_v11_pairs.next().unwrap()))
            }
            Err(pest_error) => Err(ParseError::PestError(format!("{}", pest_error))),
        }
    }
    /// Generate from pest pair
    fn from_pest_pair(pair: Pair<Rule>) -> PeerCardV11 {
        let mut currency_str = "";
        let mut node_id = NodeId(0);
        let mut issuer = None;
        let mut blockstamp = None;
        let mut endpoints = Vec::new();
        let mut sig = None;
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency_str = field.as_str(),
                Rule::node_id => node_id = NodeId(field.as_str().parse().unwrap()),
                Rule::pubkey => {
                    issuer = Some(PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(field.as_str()).unwrap(),
                    ))
                }
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // { block_id ~ "-" ~ hash }

                    let block_id: &str = inner_rules.next().unwrap().as_str();
                    let block_hash: &str = inner_rules.next().unwrap().as_str();
                    blockstamp = Some(Blockstamp {
                        id: BlockId(block_id.parse().unwrap()), // Grammar ensures that we have a digits string.
                        hash: BlockHash(Hash::from_hex(block_hash).unwrap()), // Grammar ensures that we have an hexadecimal string.
                    });
                }
                Rule::endpoint_v2 => endpoints.push(EndpointV2::from_pest_pair(field)),
                Rule::ed25519_sig => {
                    sig = Some(Sig::Ed25519(
                        ed25519::Signature::from_base64(field.as_str()).unwrap(),
                    ))
                }
                _ => panic!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        let endpoints_len = endpoints.len();
        PeerCardV11 {
            currency_name: CurrencyName(currency_str.to_owned()),
            issuer: issuer.expect("Grammar must ensure that peer v11 have valid issuer pubkey !"),
            node_id,
            blockstamp: blockstamp
                .expect("Grammar must ensure that peer v11 have valid blockstamp!"),
            endpoints,
            endpoints_str: Vec::with_capacity(endpoints_len),
            sig,
        }
    }
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
            PeerCard::V11(ref peer_v11) => peer_v11.blockstamp,
            //_ => panic!("Peer version is not supported !"),
        }
    }
    /// Get peer card issuer
    pub fn issuer(&self) -> PubKey {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.issuer,
            PeerCard::V11(ref peer_v11) => peer_v11.issuer,
            //_ => panic!("Peer version is not supported !"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use tests::keypair1;

    fn create_endpoint_v2() -> EndpointV2 {
        EndpointV2 {
            api: NetworkEndpointApi(String::from("WS2P")),
            api_version: 2,
            network_features: EndpointV2NetworkFeatures(vec![1u8]),
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
            network_features: EndpointV2NetworkFeatures(vec![1u8]),
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
            endpoints: vec![create_endpoint_v2(), create_second_endpoint_v2()],
            endpoints_str: vec![],
            sig: None,
        };
        // Sign
        let sign_result = peer_card_v11.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(peer_card_v11_raw) = sign_result {
            println!("{}", peer_card_v11_raw);
            assert_eq!(
                peer_card_v11,
                PeerCardV11::parse_from_raw(&peer_card_v11_raw)
                    .expect("Fail to parse peer card v11 !")
            )
        } else {
            panic!("fail to sign peer card : {:?}", sign_result.err().unwrap())
        }
        // Verify signature
        peer_card_v11
            .verify()
            .expect("Fail to verify PeerCardV11 !");
    }
}

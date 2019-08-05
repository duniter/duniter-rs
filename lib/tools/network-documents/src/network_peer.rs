//  Copyright (C) 2017  The Dunitrust Project Developers.
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

use crate::network_endpoint::*;
use crate::*;
use base58::ToBase58;
use dubp_documents::blockstamp::Blockstamp;
use dubp_documents::BlockNumber;
use dubp_documents::ToStringObject;
use dup_crypto::keys::text_signable::TextSignable;
use dup_crypto::keys::*;
use dup_currency_params::CurrencyName;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Block number when the peer record was created
    pub created_on: BlockNumber,
    /// Peer card binary endpoints
    pub endpoints: Vec<EndpointV2>,
    /// Peer card string endpoints
    pub endpoints_str: Vec<String>,
    /// Signature
    pub sig: Option<Sig>,
}

impl PeerCardV11 {
    /// From pest parser pair
    pub fn from_pest_pair(pair: Pair<Rule>) -> Result<PeerCardV11, TextDocumentParseError> {
        let mut currency_str = "";
        let mut node_id = NodeId(0);
        let mut issuer = None;
        let mut created_on = None;
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
                Rule::block_id => {
                    created_on = Some(BlockNumber(field.as_str().parse().unwrap())); // Grammar ensures that we have a digits string.
                }
                Rule::endpoint_v2 => endpoints.push(EndpointV2::from_pest_pair(field)?),
                Rule::ed25519_sig => {
                    sig = Some(Sig::Ed25519(
                        ed25519::Signature::from_base64(field.as_str()).unwrap(),
                    ))
                }
                _ => fatal_error!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        let endpoints_len = endpoints.len();

        Ok(PeerCardV11 {
            currency_name: CurrencyName(currency_str.to_owned()),
            issuer: issuer.expect("Grammar must ensure that peer v11 have valid issuer pubkey !"),
            node_id,
            created_on: created_on
                .expect("Grammar must ensure that peer v11 have valid field created_on !"),
            endpoints,
            endpoints_str: Vec::with_capacity(endpoints_len),
            sig,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// identity document for jsonification
pub struct PeerCardV11Stringified {
    /// Currency name
    pub currency_name: String,
    /// Peer card issuer
    pub issuer: String,
    /// Issuer node id
    pub node_id: String,
    /// Block number when the peer record was created
    pub created_on: u32,
    /// Peer card string endpoints
    pub endpoints: Vec<String>,
    /// Signature
    pub sig: String,
}

impl ToStringObject for PeerCardV11 {
    type StringObject = PeerCardV11Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> PeerCardV11Stringified {
        let mut endpoints: Vec<String> = self.endpoints.iter().map(EndpointV2::to_string).collect();
        endpoints.extend_from_slice(&self.endpoints_str);

        PeerCardV11Stringified {
            currency_name: self.currency_name.0.clone(),
            issuer: format!("{}", self.issuer),
            node_id: format!("{}", self.node_id),
            created_on: self.created_on.0,
            endpoints,
            sig: if let Some(sig) = self.sig {
                format!("{}", sig)
            } else {
                "".to_owned()
            },
        }
    }
}

impl TextSignable for PeerCardV11 {
    fn as_signable_text(&self) -> String {
        format!(
            "11:{currency}:{node_id}:{pubkey}:{created_on}\n{endpoinds}\n{endpoints_str}{nl}",
            currency = self.currency_name.0,
            node_id = format!("{}", self.node_id),
            pubkey = self.issuer.to_base58(),
            created_on = self.created_on.0,
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

impl TextDocumentParser<Rule> for PeerCard {
    type DocumentType = PeerCard;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut peer_v11_pairs = NetworkDocsParser::parse(Rule::peer_v11, doc)?;
        Ok(Self::from_versioned_pest_pair(
            11,
            peer_v11_pairs.next().unwrap(),
        )?)
    }

    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        Self::from_versioned_pest_pair(11, pair)
    }

    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            11 => Ok(PeerCard::V11(PeerCardV11::from_pest_pair(pair)?)),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
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
            created_on: self.created_on.0,
            endpoints: self.endpoints.iter().map(EndpointV2::to_string).collect(),
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
    pub created_on: u32,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Signature
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Get peer card created_on field
    pub fn created_on(&self) -> BlockNumber {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.blockstamp.id,
            PeerCard::V11(ref peer_v11) => peer_v11.created_on,
        }
    }
    /// Get peer card issuer
    pub fn issuer(&self) -> PubKey {
        match *self {
            PeerCard::V10(ref peer_v10) => peer_v10.issuer,
            PeerCard::V11(ref peer_v11) => peer_v11.issuer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::keypair1;
    use std::net::Ipv4Addr;
    use std::str::FromStr;

    fn create_endpoint_v2() -> EndpointV2 {
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
    fn create_second_endpoint_v2() -> EndpointV2 {
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

    #[test]
    fn peer_card_v11_sign_and_verify() {
        let keypair1 = keypair1();
        let mut peer_card_v11 = PeerCardV11 {
            currency_name: CurrencyName(String::from("g1")),
            issuer: PubKey::Ed25519(keypair1.public_key()),
            node_id: NodeId(0),
            created_on: BlockNumber(50),
            endpoints: vec![create_endpoint_v2(), create_second_endpoint_v2()],
            endpoints_str: vec![],
            sig: None,
        };
        // Sign
        let sign_result = peer_card_v11.sign(PrivKey::Ed25519(keypair1.private_key()));
        if let Ok(peer_card_v11_raw) = sign_result {
            println!("{}", peer_card_v11_raw);
            assert_eq!(
                PeerCard::V11(peer_card_v11.clone()),
                PeerCard::parse(&peer_card_v11_raw).expect("Fail to parse peer card v11 !")
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

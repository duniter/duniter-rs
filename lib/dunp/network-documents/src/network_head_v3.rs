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

//! Module defining the format of network heads v3 and how to handle them.

use crate::network_head::NetworkHead;
use crate::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::traits::ToStringObject;
use dubp_common_doc::{BlockHash, BlockNumber};
use dubp_currency_params::CurrencyName;
use dup_crypto::bases::b58::ToBase58;
use dup_crypto::keys::text_signable::TextSignable;
use dup_crypto::keys::*;
use pest::iterators::Pair;
use pest::Parser;
use std::cmp::Ordering;
use unwrap::unwrap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3
pub struct NetworkHeadV3 {
    /// Currency name
    pub currency_name: CurrencyName,
    /// WS2P Private configuration
    pub api_outgoing_conf: u8,
    /// WS2P Public configuration
    pub api_incoming_conf: u8,
    /// Issuer node free member rooms
    pub free_member_rooms: u8,
    /// Issuer node free mirror rooms
    pub free_mirror_rooms: u8,
    /// Issuer node id
    pub node_id: NodeId,
    /// Issuer pubkey
    pub pubkey: PubKey,
    /// Head blockstamp
    pub blockstamp: Blockstamp,
    /// Issuer node software
    pub software: String,
    /// Issuer node soft version
    pub soft_version: String,
    /// Issuer signature
    pub signature: Option<Sig>,
    /// Head step
    pub step: u8,
}

impl NetworkHeadV3 {
    /// From pest parser pair
    pub fn from_pest_pair(pair: Pair<Rule>) -> Result<NetworkHeadV3, TextDocumentParseError> {
        let mut currency_str = "";
        let mut api_outgoing_conf = 0;
        let mut api_incoming_conf = 0;
        let mut free_member_rooms = 0;
        let mut free_mirror_rooms = 0;
        let mut node_id = NodeId(0);
        let mut pubkey = None;
        let mut blockstamp = None;
        let mut software = "";
        let mut soft_version = "";
        let mut signature = None;
        let mut step = 0;
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency_str = field.as_str(),
                Rule::api_outgoing_conf => {
                    api_outgoing_conf = unwrap!(
                        field.as_str().parse(),
                        "Fail to parse Rule::api_outgoing_conf"
                    )
                }
                Rule::api_incoming_conf => {
                    api_incoming_conf = unwrap!(
                        field.as_str().parse(),
                        "Fail to parse Rule::api_incoming_conf"
                    )
                }
                Rule::free_member_rooms => {
                    free_member_rooms = unwrap!(
                        field.as_str().parse(),
                        "Fail to parse Rule::free_member_rooms"
                    )
                }
                Rule::free_mirror_rooms => {
                    free_mirror_rooms = unwrap!(
                        field.as_str().parse(),
                        "Fail to parse Rule::free_mirror_rooms"
                    )
                }
                Rule::node_id => {
                    node_id = NodeId(unwrap!(
                        field.as_str().parse(),
                        "Fail to parse Rule::node_id"
                    ))
                }
                Rule::pubkey => {
                    pubkey = Some(PubKey::Ed25519(unwrap!(
                        ed25519::PublicKey::from_base58(field.as_str()),
                        "Fail to parse Rule::pubkey"
                    )))
                }
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // { block_id ~ "-" ~ hash }

                    let block_id: &str =
                        unwrap!(inner_rules.next(), "Fail to parse Rule::blockstamp::id").as_str();
                    let block_hash: &str =
                        unwrap!(inner_rules.next(), "Fail to parse Rule::blockstamp::hash")
                            .as_str();
                    blockstamp = Some(Blockstamp {
                        id: BlockNumber(unwrap!(
                            block_id.parse(),
                            "Fail to parse Rule::blockstamp::id"
                        )), // Grammar ensures that we have a digits string.
                        hash: BlockHash(unwrap!(
                            Hash::from_hex(block_hash),
                            "Fail to parse Rule::blockstamp::hash"
                        )), // Grammar ensures that we have an hexadecimal string.
                    });
                }
                Rule::software => software = field.as_str(),
                Rule::soft_version => soft_version = field.as_str(),
                Rule::ed25519_sig => {
                    signature = Some(Sig::Ed25519(unwrap!(
                        ed25519::Signature::from_base64(field.as_str()),
                        "Fail to parse Rule::ed25519_sig"
                    )))
                }
                Rule::step => step = unwrap!(field.as_str().parse(), "Fail to parse Rule::step"),
                _ => fatal_error!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }

        Ok(NetworkHeadV3 {
            currency_name: CurrencyName(currency_str.to_owned()),
            api_outgoing_conf,
            api_incoming_conf,
            free_member_rooms,
            free_mirror_rooms,
            node_id,
            pubkey: unwrap!(
                pubkey,
                "Grammar must ensure that head v3 have valid issuer pubkey !"
            ),
            blockstamp: unwrap!(
                blockstamp,
                "Grammar must ensure that head v3 have valid blockstamp!"
            ),
            software: software.to_owned(),
            soft_version: soft_version.to_owned(),
            signature,
            step,
        })
    }
}

impl PartialOrd for NetworkHeadV3 {
    fn partial_cmp(&self, other: &NetworkHeadV3) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NetworkHeadV3 {
    fn cmp(&self, other: &NetworkHeadV3) -> Ordering {
        self.blockstamp.cmp(&other.blockstamp)
    }
}

impl TextSignable for NetworkHeadV3 {
    fn as_signable_text(&self) -> String {
        format!(
"3:{currency}:{api_outgoing_conf}:{api_incoming_conf}:{free_member_rooms}:{free_mirror_rooms}:{node_id}:{pubkey}:{blockstamp}:{software}:{soft_version}\n",
            currency = self.currency_name,
            api_outgoing_conf = self.api_outgoing_conf,
            api_incoming_conf = self.api_incoming_conf,
            free_member_rooms = self.free_member_rooms,
            free_mirror_rooms = self.free_mirror_rooms,
            node_id = format!("{}", self.node_id),
            pubkey = self.pubkey.to_base58(),
            blockstamp = self.blockstamp.to_string(),
            software = self.software,
            soft_version = self.soft_version,
        )
    }
    fn issuer_pubkey(&self) -> PubKey {
        self.pubkey
    }
    fn signature(&self) -> Option<Sig> {
        self.signature
    }
    fn set_signature(&mut self, signature: Sig) {
        self.signature = Some(signature);
    }
}

impl TextDocumentParser<Rule> for NetworkHead {
    /// Type of document generated by the parser
    type DocumentType = NetworkHead;

    fn parse(doc: &str) -> Result<NetworkHead, TextDocumentParseError> {
        let mut head_v3_pairs = NetworkDocsParser::parse(Rule::head_v3, doc)?;
        Self::from_versioned_pest_pair(
            3,
            head_v3_pairs.next().expect("Fail to parse Rule::head_v3"),
        )
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        Self::from_versioned_pest_pair(3, pair)
    }
    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<NetworkHead, TextDocumentParseError> {
        match version {
            3 => Ok(NetworkHead::V3(Box::new(NetworkHeadV3::from_pest_pair(
                pair,
            )?))),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3 for json serializer
pub struct HeadV3Stringified {
    /// Head body
    pub body: String,
    /// Issuer signature
    pub signature: Option<String>,
    /// Head step
    pub step: u8,
}

impl<'a> ToStringObject for NetworkHeadV3 {
    type StringObject = HeadV3Stringified;

    fn to_string_object(&self) -> Self::StringObject {
        let body = self.as_signable_text();
        let body_len = body.len();
        HeadV3Stringified {
            body: body.chars().take(body_len - 1).collect(),
            signature: if let Some(sig) = self.signature {
                Some(sig.to_base64())
            } else {
                None
            },
            step: self.step,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::keypair1;

    #[test]
    fn head_v3_sign_and_verify() {
        let keypair = keypair1();
        let signator =
            SignatorEnum::Ed25519(keypair.generate_signator().expect("Fail to gen signator"));
        let mut head_v3 = NetworkHeadV3 {
            currency_name: CurrencyName("g1".to_owned()),
            api_outgoing_conf: 0u8,
            api_incoming_conf: 0u8,
            free_mirror_rooms: 0u8,
            free_member_rooms: 0u8,
            node_id: NodeId(0),
            pubkey: PubKey::Ed25519(keypair.public_key()),
            blockstamp: unwrap!(Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            )),
            software: String::from("dunitrust"),
            soft_version: String::from("0.3.0-alpha3.14"),
            signature: None,
            step: 0,
        };
        // Sign
        let sign_result = head_v3.sign(&signator);
        match sign_result {
            Ok(head_v3_raw) => {
                println!("{}", head_v3_raw);
                assert_eq!(
                    NetworkHead::V3(Box::new(head_v3.clone())),
                    NetworkHead::parse(&head_v3_raw).expect("Fail to parse head v3 !")
                )
            }
            Err(e) => panic!("fail to sign head v3 : {:?}", e),
        }
        // Verify signature
        head_v3.verify().expect("HEADv3 : Invalid signature !");
    }
}

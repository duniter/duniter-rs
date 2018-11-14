//  Copyright (C) 2017  The Durs Project Developers.
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

use base58::ToBase58;
use dubp_documents::blockstamp::Blockstamp;
use dubp_documents::{BlockHash, BlockId, CurrencyName, ToJsonObject};
use dup_crypto::keys::text_signable::TextSignable;
use dup_crypto::keys::*;
use pest::iterators::Pair;
use pest::Parser;
use std::cmp::Ordering;
use *;

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

/*impl<'de> BinSignable<'de> for NetworkHeadV3 {
    fn issuer_pubkey(&self) -> PubKey {
        self.pubkey
    }
    fn signature(&self) -> Option<Sig> {
        self.signature
    }
    fn set_signature(&mut self, signature: Sig) {
        self.signature = Some(signature);
    }
}*/

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

impl NetworkHeadV3 {
    /// parse from raw ascii format
    pub fn parse_from_raw(raw_peer: &str) -> Result<NetworkHeadV3, ParseError> {
        match NetworkDocsParser::parse(Rule::head_v3, raw_peer) {
            Ok(mut head_v3_pairs) => {
                Ok(NetworkHeadV3::from_pest_pair(head_v3_pairs.next().unwrap()))
            }
            Err(pest_error) => Err(ParseError::PestError(format!("{}", pest_error))),
        }
    }
    /// Generate from pest pair
    pub fn from_pest_pair(pair: Pair<Rule>) -> NetworkHeadV3 {
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
                Rule::api_outgoing_conf => api_outgoing_conf = field.as_str().parse().unwrap(),
                Rule::api_incoming_conf => api_incoming_conf = field.as_str().parse().unwrap(),
                Rule::free_member_rooms => free_member_rooms = field.as_str().parse().unwrap(),
                Rule::free_mirror_rooms => free_mirror_rooms = field.as_str().parse().unwrap(),
                Rule::node_id => node_id = NodeId(field.as_str().parse().unwrap()),
                Rule::pubkey => {
                    pubkey = Some(PubKey::Ed25519(
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
                Rule::software => software = field.as_str(),
                Rule::soft_version => soft_version = field.as_str(),
                Rule::ed25519_sig => {
                    signature = Some(Sig::Ed25519(
                        ed25519::Signature::from_base64(field.as_str()).unwrap(),
                    ))
                }
                Rule::step => step = field.as_str().parse().unwrap(),
                _ => panic!("unexpected rule: {:?}", field.as_rule()), // Grammar ensures that we never reach this line
            }
        }
        NetworkHeadV3 {
            currency_name: CurrencyName(currency_str.to_owned()),
            api_outgoing_conf,
            api_incoming_conf,
            free_member_rooms,
            free_mirror_rooms,
            node_id,
            pubkey: pubkey.expect("Grammar must ensure that head v3 have valid issuer pubkey !"),
            blockstamp: blockstamp
                .expect("Grammar must ensure that head v3 have valid blockstamp!"),
            software: software.to_owned(),
            soft_version: soft_version.to_owned(),
            signature,
            step,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3 for json serializer
pub struct JsonHeadV3 {
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
    /// Issuer key algorithm
    pub algorithm: KeysAlgo,
    /// Issuer pubkey
    pub pubkey: String,
    /// Head blockstamp
    pub blockstamp: String,
    /// Issuer node software
    pub software: String,
    /// Issuer node soft version
    pub soft_version: String,
    /// Issuer signature
    pub signature: Option<String>,
    /// Head step
    pub step: u8,
}

impl<'a> ToJsonObject for NetworkHeadV3 {
    type JsonObject = JsonHeadV3;

    fn to_json_object(&self) -> Self::JsonObject {
        JsonHeadV3 {
            api_outgoing_conf: self.api_outgoing_conf,
            api_incoming_conf: self.api_incoming_conf,
            free_member_rooms: self.free_member_rooms,
            free_mirror_rooms: self.free_mirror_rooms,
            node_id: self.node_id,
            algorithm: self.pubkey.algo(),
            pubkey: self.pubkey.to_base58(),
            blockstamp: self.blockstamp.to_string(),
            software: self.software.clone(),
            soft_version: self.soft_version.clone(),
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
    use tests::keypair1;

    #[test]
    fn head_v3_sign_and_verify() {
        let keypair = keypair1();
        let mut head_v3 = NetworkHeadV3 {
            currency_name: CurrencyName("g1".to_owned()),
            api_outgoing_conf: 0u8,
            api_incoming_conf: 0u8,
            free_mirror_rooms: 0u8,
            free_member_rooms: 0u8,
            node_id: NodeId(0),
            pubkey: PubKey::Ed25519(keypair.public_key()),
            blockstamp: Blockstamp::from_string(
                "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
            )
            .unwrap(),
            software: String::from("durs"),
            soft_version: String::from("0.1.0-a0.1"),
            signature: None,
            step: 0,
        };
        // Sign
        let sign_result = head_v3.sign(PrivKey::Ed25519(keypair.private_key()));
        if let Ok(head_v3_raw) = sign_result {
            println!("{}", head_v3_raw);
            assert_eq!(
                head_v3,
                NetworkHeadV3::parse_from_raw(&head_v3_raw).expect("Fail to parse head v3 !")
            )
        } else {
            panic!("fail to sign head v3 : {:?}", sign_result.err().unwrap())
        }
        // Verify signature
        head_v3.verify().expect("HEADv3 : Invalid signature !");
    }
}

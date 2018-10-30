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
use duniter_crypto::keys::bin_signable::BinSignable;
use duniter_crypto::keys::*;
use duniter_documents::blockstamp::Blockstamp;
use serde_json;
use std::cmp::Ordering;
use NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3
pub struct NetworkHeadV3Container {
    /// Head step
    pub step: u8,
    /// head body
    pub body: NetworkHeadV3,
}

impl PartialOrd for NetworkHeadV3Container {
    fn partial_cmp(&self, other: &NetworkHeadV3Container) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NetworkHeadV3Container {
    fn cmp(&self, other: &NetworkHeadV3Container) -> Ordering {
        self.body.cmp(&other.body)
    }
}

impl NetworkHeadV3Container {
    /// Convert to JSON String
    pub fn to_json_head(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string_pretty(&JsonHeadV3 {
            api_outgoing_conf: self.body.api_outgoing_conf,
            api_incoming_conf: self.body.api_incoming_conf,
            free_mirror_rooms: self.body.free_mirror_rooms,
            low_priority_rooms: self.body.low_priority_rooms,
            node_id: self.body.node_id,
            algorithm: self.body.pubkey.algo(),
            pubkey: self.body.pubkey.to_base58(),
            blockstamp: self.body.blockstamp.to_string(),
            software: &self.body.software,
            soft_version: &self.body.soft_version,
            signature: if let Some(sig) = self.body.signature {
                Some(sig.to_base64())
            } else {
                None
            },
            step: self.step,
        })?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3
pub struct NetworkHeadV3 {
    /// WS2P Private configuration
    pub api_outgoing_conf: u8,
    /// WS2P Public configuration
    pub api_incoming_conf: u8,
    /// Issuer node free mirror rooms
    pub free_mirror_rooms: u8,
    /// Issuer node free "low priority" rooms
    pub low_priority_rooms: u8,
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

impl<'de> BinSignable<'de> for NetworkHeadV3 {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3 for json serializer
pub struct JsonHeadV3<'a> {
    /// WS2P Private configuration
    pub api_outgoing_conf: u8,
    /// WS2P Public configuration
    pub api_incoming_conf: u8,
    /// Issuer node free mirror rooms
    pub free_mirror_rooms: u8,
    /// Issuer node free "low priority" rooms
    pub low_priority_rooms: u8,
    /// Issuer node id
    pub node_id: NodeId,
    /// Issuer key algorithm
    pub algorithm: KeysAlgo,
    /// Issuer pubkey
    pub pubkey: String,
    /// Head blockstamp
    pub blockstamp: String,
    /// Issuer node software
    pub software: &'a str,
    /// Issuer node soft version
    pub soft_version: &'a str,
    /// Issuer signature
    pub signature: Option<String>,
    /// Head step
    pub step: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_crypto::keys::bin_signable::BinSignable;
    use tests::bincode::deserialize;
    use tests::keypair1;

    #[test]
    fn head_v3_sign_and_verify() {
        let mut head_v3 = NetworkHeadV3Container {
            step: 0,
            body: NetworkHeadV3 {
                api_outgoing_conf: 0u8,
                api_incoming_conf: 0u8,
                free_mirror_rooms: 0u8,
                low_priority_rooms: 0u8,
                node_id: NodeId(0),
                pubkey: PubKey::Ed25519(keypair1().public_key()),
                blockstamp: Blockstamp::from_string(
                    "50-000005B1CEB4EC5245EF7E33101A330A1C9A358EC45A25FC13F78BB58C9E7370",
                )
                .unwrap(),
                software: String::from("durs"),
                soft_version: String::from("0.1.0-a0.1"),
                signature: None,
            },
        };
        // Sign
        let sign_result = head_v3
            .body
            .sign(PrivKey::Ed25519(keypair1().private_key()));
        if let Ok(head_v3_body_bytes) = sign_result {
            let deser_head_v3_body: NetworkHeadV3 =
                deserialize(&head_v3_body_bytes).expect("Fail to deserialize PeerCardV11 !");
            assert_eq!(head_v3.body, deser_head_v3_body,)
        } else {
            panic!("failt to sign head v3 : {:?}", sign_result.err().unwrap())
        }
        // Verify signature
        head_v3.body.verify().expect("HEADv3 : Invalid signature !");
        //let json_head_v3 = head_v3.to_json_head().expect("Fail to serialize HEAD v3 !");
        //println!("{}", json_head_v3);
        //panic!();
    }
}

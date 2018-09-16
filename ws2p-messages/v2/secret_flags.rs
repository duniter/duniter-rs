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

use duniter_crypto::keys::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// WS2Pv2SecretFlags
pub struct WS2Pv2SecretFlags(Vec<u8>);

impl WS2Pv2SecretFlags {
    /// Return true if all flags are disabled (or if it's really empty).
    pub fn is_empty(&self) -> bool {
        for byte in &self.0 {
            if *byte > 0u8 {
                return false;
            }
        }
        true
    }
    /// Check flag LOW_FLOW_DEMAND
    pub fn _low_flow_demand(&self) -> bool {
        self.0[0] | 0b1111_1110 == 255u8
    }
    /// Check flag MEMBER_PUBKEY
    pub fn member_pubkey(&self) -> bool {
        self.0[0] | 0b1111_1101 == 255u8
    }
    /// Check flag MEMBER_PROOF
    pub fn member_proof(&self) -> bool {
        self.0[0] | 0b1111_1011 == 255u8
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// WS2Pv2SecretFlagsMsg
pub struct WS2Pv2SecretFlagsMsg {
    /// Secret flags
    pub secret_flags: WS2Pv2SecretFlags,
    ///
    pub member_pubkey: Option<PubKey>,
    /// Proof that the sender node is a member (Signature of the challenge send by other node in their CONNECT message.)
    pub member_proof: Option<Sig>,
}

impl Default for WS2Pv2SecretFlagsMsg {
    fn default() -> Self {
        WS2Pv2SecretFlagsMsg {
            secret_flags: WS2Pv2SecretFlags(vec![]),
            member_pubkey: None,
            member_proof: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use tests::*;

    #[test]
    fn test_ws2p_message_secret_flags() {
        let keypair1 = keypair1();
        let challenge = Hash::random();
        let msg = WS2Pv2SecretFlagsMsg {
            secret_flags: WS2Pv2SecretFlags(vec![6u8]),
            member_pubkey: Some(PubKey::Ed25519(keypair1.public_key())),
            member_proof: Some(Sig::Ed25519(keypair1.private_key().sign(&challenge.0))),
        };
        test_ws2p_message(WS2Pv0MessagePayload::SecretFlags(msg));
    }
}

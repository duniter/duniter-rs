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

/*
impl BinMessage for WS2Pv2SecretFlagsMsg {
    type ReadBytesError = WS2Pv0MsgPayloadContentParseError;
    fn from_bytes(datas: &[u8]) -> Result<Self, Self::ReadBytesError> {
        // Read flags_size
        if datas.is_empty() {
            return Err(WS2Pv0MsgPayloadContentParseError::TooShort("Empty datas !"));
        }
        let mut index = 0;
        let flags_size = datas[0] as usize;
        index += 1;
        // Read secret_flags
        if datas.len() < flags_size + index {
            return Err(WS2Pv0MsgPayloadContentParseError::TooShort("secret_flags"));
        }
        let secret_flags = WS2Pv2SecretFlags(datas[index..index + flags_size].to_vec());
        index += flags_size;
        // Read member_pubkey
        let (member_pubkey, key_algo) = if secret_flags.member_pubkey() {
            if datas.len() < index + 2 {
                return Err(WS2Pv0MsgPayloadContentParseError::TooShort(
                    "member_pubkey: pubkey_box size",
                ));
            }
            // Read pubkey_box size
            let pubkey_box_size = u16::read_u16_be(&datas[index..index + 2])? as usize;
            index += 2;
            if datas.len() < index + pubkey_box_size {
                return Err(WS2Pv0MsgPayloadContentParseError::TooShort(
                    "member_pubkey: pubkey_box",
                ));
            }
            // Read pubkey_box
            index += pubkey_box_size;
            let (member_pubkey, key_algo) =
                pubkey_box::read_pubkey_box(&datas[index - pubkey_box_size..index])?;
            (Some(member_pubkey), key_algo)
        } else {
            (None, 0u8)
        };
        // Read member_proof
        let member_proof = if member_pubkey.is_some() && secret_flags.member_proof() {
            if datas.len() < index + 2 {
                return Err(WS2Pv0MsgPayloadContentParseError::TooShort(
                    "member_proof: sig_box size",
                ));
            }
            // Read sig_box size
            let sig_box_size = u16::read_u16_be(&datas[index..index + 2])? as usize;
            index += 2;
            if datas.len() < index + sig_box_size {
                return Err(WS2Pv0MsgPayloadContentParseError::TooShort(
                    "member_proof: sig_box",
                ));
            }
            // Read sig_box
            index += sig_box_size;
            Some(sig_box::read_sig_box(
                &datas[index - sig_box_size..index],
                key_algo,
            )?)
        } else {
            None
        };
        Ok(WS2Pv2SecretFlagsMsg {
            secret_flags,
            member_pubkey,
            member_proof,
        })
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        // Compute buffer size
        let secret_flags_size = if !self.secret_flags.is_empty() {
            self.secret_flags.0.len()
        } else {
            0
        };
        let member_pubkey_size = if let Some(ref member_pubkey) = self.member_pubkey {
            member_pubkey.size_in_bytes()
        } else {
            0
        };
        let member_proof_size = if let Some(ref member_proof) = self.member_proof {
            member_proof.size_in_bytes()
        } else {
            0
        };
        let buffer_size = 1 + secret_flags_size + member_pubkey_size + member_proof_size;
        // Allocate buffer
        let mut buffer = Vec::with_capacity(buffer_size);
        // Write secret_flags_size
        buffer.push(secret_flags_size as u8);
        // Write secret_flags
        if secret_flags_size > 0 {
            buffer.append(&mut self.secret_flags.0.clone());
        }
        // Write member_pubkey
        if let Some(ref member_pubkey) = self.member_pubkey {
            pubkey_box::write_pubkey_box(&mut buffer, *member_pubkey)
                .expect("WS2Pv2SecretFlagsMsg : fail to binarize member_pubkey !");
        };
        // Write member_proof
        if let Some(ref member_proof) = self.member_proof {
            sig_box::write_sig_box(&mut buffer, *member_proof)
                .expect("WS2Pv2SecretFlagsMsg : fail  to binarize member_proof !");
        };
        buffer
    }
}*/

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

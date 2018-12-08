//  Copyright (C) 2018  The Durs Project Developers.
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

//! Generic code for signing data in binary format

use super::*;
use crate::hashs::Hash;
use bincode;
use serde::{Deserialize, Serialize};

/// Signatureable in binary format
pub trait BinSignable<'de>: Serialize + Deserialize<'de> {
    /// Return message issuer pubkey
    fn issuer_pubkey(&self) -> PubKey;
    /// Return true if message store is hash
    fn store_hash(&self) -> bool {
        false
    }
    /// Return message hash
    fn hash(&self) -> Option<Hash> {
        None
    }
    /// Change hash (redefine ly by messages with hash field)
    fn set_hash(&mut self, _hash: Hash) {}
    /// Return message signature
    fn signature(&self) -> Option<Sig>;
    /// Change signature
    fn set_signature(&mut self, _signature: Sig);
    /// Compute hash
    fn compute_hash(&self) -> Result<(Hash, Vec<u8>), bincode::Error> {
        let mut bin_msg = bincode::serialize(&self)?;
        let sig_size = bincode::serialized_size(&self.signature())?;
        let bin_msg_len = bin_msg.len();
        bin_msg.truncate(bin_msg_len - (sig_size as usize));
        if self.store_hash() {
            bin_msg.pop(); // Delete hash: None
        }
        // Compute hash of binary datas without signature
        let hash = Hash::compute(&bin_msg);
        Ok((hash, bin_msg))
    }
    /// Sign bin message
    fn sign(&mut self, priv_key: PrivKey) -> Result<Vec<u8>, SignError> {
        if self.signature().is_some() {
            return Err(SignError::AlreadySign());
        }
        match self.issuer_pubkey() {
            PubKey::Ed25519(_) => match priv_key {
                PrivKey::Ed25519(priv_key) => {
                    let (hash, mut bin_msg) = self.compute_hash().expect("Fail to compute hash !");
                    self.set_hash(hash);
                    let bin_sig = priv_key.sign(&hash.0);
                    let sig = Sig::Ed25519(bin_sig);
                    self.set_signature(sig);
                    if self.hash().is_some() {
                        bin_msg.extend_from_slice(
                            &bincode::serialize(&Some(hash)).expect("Fail to binarize hash !"),
                        );
                    }
                    bin_msg.extend_from_slice(
                        &bincode::serialize(&Some(sig)).expect("Fail to binarize sig !"),
                    );
                    Ok(bin_msg)
                }
                _ => Err(SignError::WrongAlgo()),
            },
            _ => Err(SignError::WrongAlgo()),
        }
    }
    /// Check signature of bin message
    fn verify(&self) -> Result<(), SigError> {
        if let Some(signature) = self.signature() {
            match self.issuer_pubkey() {
                PubKey::Ed25519(pubkey) => match signature {
                    Sig::Ed25519(sig) => {
                        let (hash, _) = if let Some(hash) = self.hash() {
                            (hash, vec![])
                        } else {
                            self.compute_hash()?
                        };
                        if pubkey.verify(&hash.0, &sig) {
                            Ok(())
                        } else {
                            Err(SigError::InvalidSig())
                        }
                    }
                    _ => Err(SigError::NotSameAlgo()),
                },
                _ => Err(SigError::NotSameAlgo()),
            }
        } else {
            Err(SigError::NotSig())
        }
    }
}

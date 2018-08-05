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

//! Defined all aspects of the inter-node network that concern all modules and are therefore independent of one implementation or another of this network layer.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

//#[cfg(test)]
//#[macro_use]
//extern crate pretty_assertions;

extern crate byteorder;
extern crate crypto;
extern crate duniter_crypto;

pub mod pubkey_box;
pub mod sig_box;
pub mod u16;
pub mod u32;

use duniter_crypto::hashs::*;
use duniter_crypto::keys::*;

/// BinMessage := Message in binary format.
pub trait BinMessage: Sized {
    /// ReadBytesError
    type ReadBytesError;
    /// Create Self from bytes slice
    fn from_bytes(&[u8]) -> Result<Self, Self::ReadBytesError>;
    /// Convert Self to bytes vector
    fn to_bytes_vector(&self) -> Vec<u8>;
}

/// Signatureable bin message
pub trait BinMessageSignable: BinMessage {
    /// Return message issuer pubkey
    fn issuer_pubkey(&self) -> PubKey {
        PubKey::default()
    }
    /// Return message hash
    fn hash(&self) -> Option<Hash> {
        None
    }
    /// Change hash (redefine ly by messages with hash field)
    fn set_hash(&mut self, _hash: Hash) {}
    /// Return message signature
    fn signature(&self) -> Option<Sig> {
        None
    }
    /// Store signature
    fn set_signature(&mut self, _signature: Sig);
    /// Compute hash
    fn compute_hash(&self) -> (Vec<u8>, Hash) {
        let bin_msg = self.to_bytes_vector();
        // Compute hash
        let hash = Hash::compute(&bin_msg);
        (bin_msg, hash)
    }
    /// Sign bin message
    fn sign(&mut self, priv_key: PrivKey) -> Result<Vec<u8>, SignError> {
        if self.signature().is_some() {
            return Err(SignError::AlreadySign());
        }
        match self.issuer_pubkey() {
            PubKey::Ed25519(_) => match priv_key {
                PrivKey::Ed25519(priv_key) => {
                    let (mut bin_msg, hash) = self.compute_hash();
                    self.set_hash(hash);
                    let bin_sig = priv_key.sign(&hash.0);
                    let sig = Sig::Ed25519(bin_sig);
                    self.set_signature(sig);
                    if self.hash().is_some() {
                        bin_msg.extend_from_slice(&hash.0);
                    }
                    sig_box::write_sig_box(&mut bin_msg, sig).expect("Fail to binarize sig !");
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
                        let (_, hash) = if let Some(hash) = self.hash() {
                            (vec![], hash)
                        } else {
                            self.compute_hash()
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

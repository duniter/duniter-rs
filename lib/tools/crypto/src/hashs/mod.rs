//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! Provide wrappers for cryptographic hashs

use crate::bases::*;
use rand::{thread_rng, Rng};
use std::fmt::{Debug, Display, Error, Formatter};

#[cfg(not(all(unix, any(target_arch = "x86", target_arch = "x86_64"))))]
use crypto::digest::Digest;
#[cfg(not(all(unix, any(target_arch = "x86", target_arch = "x86_64"))))]
use crypto::sha2::Sha256;
#[cfg(all(unix, any(target_arch = "x86", target_arch = "x86_64")))]
use sha2::{Digest, Sha256};

/// A hash wrapper.
///
/// A hash is often provided as string composed of 64 hexadecimal character (0 to 9 then A to F).
#[derive(Copy, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
pub struct Hash(pub [u8; 32]);

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_hex())
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Hash({})", self)
    }
}

impl Default for Hash {
    fn default() -> Hash {
        Hash([0; 32])
    }
}

impl Hash {
    /// Hash size (in bytes).
    pub const SIZE_IN_BYTES: usize = 32;

    /// Generate a random Hash
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let mut hash_bytes = Vec::with_capacity(32);
        for _ in 0..32 {
            hash_bytes.push(rng.gen::<u8>());
        }
        let mut hash_bytes_arr = [0; 32];
        hash_bytes_arr.copy_from_slice(&hash_bytes);
        Hash(hash_bytes_arr)
    }

    #[cfg(all(unix, any(target_arch = "x86", target_arch = "x86_64")))]
    /// Compute hash of any binary datas
    pub fn compute(datas: &[u8]) -> Hash {
        let hasher = Sha256::new();
        let mut hash = Hash::default();
        hash.0
            .copy_from_slice(hasher.chain(datas).result().as_slice());
        hash
    }
    #[cfg(all(unix, any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline]
    /// Compute hash of a string
    pub fn compute_str(str_datas: &str) -> Hash {
        Hash::compute(str_datas.as_bytes())
    }
    #[cfg(not(all(unix, any(target_arch = "x86", target_arch = "x86_64"))))]
    /// Compute hash of any binary datas
    pub fn compute(datas: &[u8]) -> Hash {
        let mut sha = Sha256::new();
        sha.input(datas);
        let mut hash_buffer = [0u8; 32];
        sha.result(&mut hash_buffer);
        Hash(hash_buffer)
    }
    #[cfg(not(all(unix, any(target_arch = "x86", target_arch = "x86_64"))))]
    /// Compute hash of a string
    pub fn compute_str(str_datas: &str) -> Hash {
        let mut sha256 = Sha256::new();
        sha256.input_str(&str_datas);
        Hash::from_hex(&sha256.result_str()).expect("Sha256 result must be an hexa string !")
    }

    /// Convert Hash into bytes vector
    pub fn to_bytes_vector(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Convert a `Hash` to an hex string.
    pub fn to_hex(&self) -> String {
        let strings: Vec<String> = self.0.iter().map(|b| format!("{:02X}", b)).collect();

        strings.join("")
    }

    /// Convert a hex string in a `Hash`.
    ///
    /// The hex string must only contains hex characters
    /// and produce a 32 bytes value.
    #[inline]
    pub fn from_hex(text: &str) -> Result<Hash, BaseConvertionError> {
        Ok(Hash(b16::str_hex_to_32bytes(text)?))
    }
}

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

//! Provide wrappers for cryptographic hashs

use crate::keys::BaseConvertionError;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rand::{thread_rng, Rng};
use std::fmt::{Debug, Display, Error, Formatter};

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

    /// Compute hash of any binary datas
    pub fn compute(datas: &[u8]) -> Hash {
        let mut sha = Sha256::new();
        sha.input(datas);
        let mut hash_buffer = [0u8; 32];
        sha.result(&mut hash_buffer);
        Hash(hash_buffer)
    }

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
    pub fn from_hex(text: &str) -> Result<Hash, BaseConvertionError> {
        if text.len() != 64 {
            Err(BaseConvertionError::InvalidKeyLendth(text.len(), 64))
        } else {
            let mut hash = Hash([0u8; 32]);

            let chars: Vec<char> = text.chars().collect();

            for i in 0..64 {
                if i % 2 != 0 {
                    continue;
                }

                let byte1 = chars[i].to_digit(16);
                let byte2 = chars[i + 1].to_digit(16);

                if byte1.is_none() {
                    return Err(BaseConvertionError::InvalidCharacter(chars[i], i));
                } else if byte2.is_none() {
                    return Err(BaseConvertionError::InvalidCharacter(chars[i + 1], i + 1));
                }

                let byte1 = byte1.unwrap() as u8;
                let byte2 = byte2.unwrap() as u8;

                let byte = (byte1 << 4) | byte2;
                hash.0[i / 2] = byte;
            }

            Ok(hash)
        }
    }
}

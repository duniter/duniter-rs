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

//! Implements the Duniter Documents Protocol.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate base58;
extern crate base64;
extern crate crypto;
extern crate duniter_crypto;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
extern crate regex;

use std::fmt::{Debug, Display, Error, Formatter};

use duniter_crypto::keys::BaseConvertionError;

pub mod blockchain;

/// A block Id.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl Display for BlockId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

/// A hash wrapper.
///
/// A hash is often provided as string composed of 64 hexadecimal character (0 to 9 then A to F).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
        let default: [u8; 32] = [0; 32];
        Hash(default)
    }
}

impl Hash {
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

/// Wrapper of a block hash.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockHash(pub Hash);

impl Display for BlockHash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0.to_hex())
    }
}

impl Debug for BlockHash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "BlockHash({})", self)
    }
}

/// Type of errors for [`BlockUId`] parsing.
///
/// [`BlockUId`]: struct.BlockUId.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockUIdParseError {
    /// Given string have invalid format
    InvalidFormat(),
    /// [`BlockId`](struct.BlockHash.html) part is not a valid number.
    InvalidBlockId(),
    /// [`BlockHash`](struct.BlockHash.html) part is not a valid hex number.
    InvalidBlockHash(),
}

/// A blockstamp (Unique ID).
///
/// It's composed of the [`BlockId`] and
/// the [`BlockHash`] of the block.
///
/// Thanks to blockchain immutability and frequent block production, it can
/// be used to date information.
///
/// [`BlockId`]: struct.BlockId.html
/// [`BlockHash`]: struct.BlockHash.html
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Blockstamp {
    /// Block Id.
    pub id: BlockId,
    /// Block hash.
    pub hash: BlockHash,
}

impl Display for Blockstamp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}-{}", self.id, self.hash)
    }
}

impl Debug for Blockstamp {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "BlockUId({})", self)
    }
}

impl Default for Blockstamp {
    fn default() -> Blockstamp {
        Blockstamp {
            id: BlockId(0),
            hash: BlockHash(Hash::default()),
        }
    }
}

impl Blockstamp {
    /// Create a `BlockUId` from a text.
    pub fn from_string(src: &str) -> Result<Blockstamp, BlockUIdParseError> {
        let mut split = src.split('-');

        if split.clone().count() != 2 {
            Err(BlockUIdParseError::InvalidFormat())
        } else {
            let id = split.next().unwrap().parse::<u32>();
            let hash = Hash::from_hex(split.next().unwrap());

            if id.is_err() {
                Err(BlockUIdParseError::InvalidBlockId())
            } else if hash.is_err() {
                Err(BlockUIdParseError::InvalidBlockHash())
            } else {
                Ok(Blockstamp {
                    id: BlockId(id.unwrap()),
                    hash: BlockHash(hash.unwrap()),
                })
            }
        }
    }

    /// Convert a `BlockUId` to its text format.
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

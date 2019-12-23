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

//! Wrapper for blockstamp

use crate::{BlockHash, BlockNumber};
use dup_crypto::bases::BaseConvertionError;
use dup_crypto::hashs::Hash;
use failure::Fail;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter};

/// Type of errors for [`Blockstamp`] parsing.
///
/// [`Blockstamp`]: struct.Blockstamp.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, Fail)]
pub enum BlockstampParseError {
    /// Given string have invalid format
    #[fail(display = "Given bytes with invalid length")]
    InvalidLen,
    /// Given string have invalid format
    #[fail(display = "Given string have invalid format")]
    InvalidFormat(),
    /// [`BlockNumber`](struct.BlockHash.html) part is not a valid number.
    #[fail(display = "BlockNumber part is not a valid number.")]
    InvalidBlockNumber(),
    /// [`BlockHash`](struct.BlockHash.html) part is not a valid hex number.
    #[fail(display = "BlockHash part is not a valid hex number.")]
    InvalidBlockHash(BaseConvertionError),
}

impl From<BaseConvertionError> for BlockstampParseError {
    fn from(e: BaseConvertionError) -> Self {
        BlockstampParseError::InvalidBlockHash(e)
    }
}

/// A blockstamp (Unique ID).
///
/// It's composed of the [`BlockNumber`] and
/// the [`BlockHash`] of the block.
///
/// Thanks to blockchain immutability and frequent block production, it can
/// be used to date information.
///
/// [`BlockNumber`]: struct.BlockNumber.html
/// [`BlockHash`]: struct.BlockHash.html

#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct Blockstamp {
    /// Block Id.
    pub id: BlockNumber,
    /// Block hash.
    pub hash: BlockHash,
}

/// Previous blockstamp (BlockNumber-1, previous_hash)
pub type PreviousBlockstamp = Blockstamp;

impl Blockstamp {
    /// Blockstamp size (in bytes).
    pub const SIZE_IN_BYTES: usize = 36;
}

impl Into<Vec<u8>> for Blockstamp {
    fn into(self) -> Vec<u8> {
        let mut bytes = [0u8; Self::SIZE_IN_BYTES];

        bytes[..4].copy_from_slice(&self.id.0.to_be_bytes()[..4]);
        bytes[4..Self::SIZE_IN_BYTES].copy_from_slice(&(self.hash.0).0[..]);

        bytes.to_vec()
    }
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
            id: BlockNumber(0),
            hash: BlockHash(Hash::default()),
        }
    }
}

impl PartialOrd for Blockstamp {
    fn partial_cmp(&self, other: &Blockstamp) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Blockstamp {
    fn cmp(&self, other: &Blockstamp) -> Ordering {
        if self.id == other.id {
            self.hash.cmp(&other.hash)
        } else {
            self.id.cmp(&other.id)
        }
    }
}

#[derive(Debug)]
/// Error when converting a byte vector to Blockstamp
pub enum ReadBytesBlockstampError {
    /// Bytes vector is too short
    TooShort(),
    /// Bytes vector is too long
    TooLong(),
    /// IoError
    IoError(std::io::Error),
}

impl From<std::io::Error> for ReadBytesBlockstampError {
    fn from(e: std::io::Error) -> Self {
        ReadBytesBlockstampError::IoError(e)
    }
}

impl Blockstamp {
    /// Create a `Blockstamp` from bytes.
    pub fn from_bytes(src: &[u8]) -> Result<Blockstamp, BlockstampParseError> {
        if src.len() != Blockstamp::SIZE_IN_BYTES {
            Err(BlockstampParseError::InvalidLen)
        } else {
            let mut id_bytes = [0u8; 4];
            id_bytes.copy_from_slice(&src[..4]);
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(&src[4..]);
            Ok(Blockstamp {
                id: BlockNumber(u32::from_be_bytes(id_bytes)),
                hash: BlockHash(Hash(hash_bytes)),
            })
        }
    }

    /// Create a `Blockstamp` from a text.
    pub fn from_string(src: &str) -> Result<Blockstamp, BlockstampParseError> {
        let mut split = src.split('-');

        match (split.next(), split.next(), split.next()) {
            (Some(id), Some(hash), None) => {
                let hash = Hash::from_hex(hash)?;

                if let Ok(id) = id.parse::<u32>() {
                    Ok(Blockstamp {
                        id: BlockNumber(id),
                        hash: BlockHash(hash),
                    })
                } else {
                    Err(BlockstampParseError::InvalidBlockNumber())
                }
            }
            _ => Err(BlockstampParseError::InvalidFormat()),
        }
    }
}

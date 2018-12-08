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

//! Wrapper for blockstamp

use crate::*;

/// Type of errors for [`BlockUId`] parsing.
///
/// [`BlockUId`]: struct.BlockUId.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockstampParseError {
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

#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct Blockstamp {
    /// Block Id.
    pub id: BlockId,
    /// Block hash.
    pub hash: BlockHash,
}

/// Previous blockstamp (BlockId-1, previous_hash)
pub type PreviousBlockstamp = Blockstamp;

impl Blockstamp {
    /// Blockstamp size (in bytes).
    pub const SIZE_IN_BYTES: usize = 36;
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
    IoError(::std::io::Error),
}

impl From<::std::io::Error> for ReadBytesBlockstampError {
    fn from(e: ::std::io::Error) -> Self {
        ReadBytesBlockstampError::IoError(e)
    }
}

impl Blockstamp {
    /// Create a `BlockUId` from a text.
    pub fn from_string(src: &str) -> Result<Blockstamp, BlockstampParseError> {
        let mut split = src.split('-');

        if split.clone().count() != 2 {
            Err(BlockstampParseError::InvalidFormat())
        } else {
            let id = split.next().unwrap().parse::<u32>();
            let hash = Hash::from_hex(split.next().unwrap());

            if id.is_err() {
                Err(BlockstampParseError::InvalidBlockId())
            } else if hash.is_err() {
                Err(BlockstampParseError::InvalidBlockHash())
            } else {
                Ok(Blockstamp {
                    id: BlockId(id.unwrap()),
                    hash: BlockHash(
                        hash.expect("Try to get hash of an uncompleted or reduce block !"),
                    ),
                })
            }
        }
    }

    /// Convert a `BlockUId` to its text format.
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

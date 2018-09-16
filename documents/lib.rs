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
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate base58;
extern crate base64;
extern crate byteorder;
extern crate crypto;
extern crate duniter_crypto;
extern crate linked_hash_map;
extern crate regex;
extern crate serde;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use currencies_codes::*;
use duniter_crypto::hashs::Hash;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter};
use std::io::Cursor;
use std::mem;

pub mod blockchain;
mod currencies_codes;

/// Currency name
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize, Hash)]
pub struct CurrencyName(pub String);

impl Default for CurrencyName {
    fn default() -> CurrencyName {
        CurrencyName(String::from("default_currency"))
    }
}

impl Display for CurrencyName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

/// CurrencyCodeError
#[derive(Debug)]
pub enum CurrencyCodeError {
    /// UnknowCurrencyCode
    UnknowCurrencyCode(),
    /// IoError
    IoError(::std::io::Error),
    /// UnknowCurrencyName
    UnknowCurrencyName(),
}

impl From<::std::io::Error> for CurrencyCodeError {
    fn from(error: ::std::io::Error) -> Self {
        CurrencyCodeError::IoError(error)
    }
}

impl CurrencyName {
    /// Convert bytes to CurrencyName
    pub fn from(currency_code: [u8; 2]) -> Result<Self, CurrencyCodeError> {
        let mut currency_code_bytes = Cursor::new(currency_code.to_vec());
        let currency_code = currency_code_bytes.read_u16::<BigEndian>()?;
        Self::from_u16(currency_code)
    }
    /// Convert u16 to CurrencyName
    pub fn from_u16(currency_code: u16) -> Result<Self, CurrencyCodeError> {
        match currency_code {
            tmp if tmp == *CURRENCY_NULL => Ok(CurrencyName(String::from(""))),
            tmp if tmp == *CURRENCY_G1 => Ok(CurrencyName(String::from("g1"))),
            tmp if tmp == *CURRENCY_G1_TEST => Ok(CurrencyName(String::from("g1-test"))),
            _ => Err(CurrencyCodeError::UnknowCurrencyCode()),
        }
    }
    /// Convert CurrencyName to bytes
    pub fn to_bytes(&self) -> Result<[u8; 2], CurrencyCodeError> {
        let currency_code = match self.0.as_str() {
            "g1" => *CURRENCY_G1,
            "g1-test" => *CURRENCY_G1_TEST,
            _ => return Err(CurrencyCodeError::UnknowCurrencyName()),
        };
        let mut buffer = [0u8; mem::size_of::<u16>()];
        buffer
            .as_mut()
            .write_u16::<BigEndian>(currency_code)
            .expect("Unable to write");
        Ok(buffer)
    }
}

/// A block Id.
#[derive(Copy, Clone, Debug, Deserialize, Ord, PartialEq, PartialOrd, Eq, Hash, Serialize)]
pub struct BlockId(pub u32);

impl Display for BlockId {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

/// Wrapper of a block hash.
#[derive(Copy, Clone, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
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

/*
impl BinMessage for Blockstamp {
    type ReadBytesError = ReadBytesBlockstampError;
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::ReadBytesError> {
        if bytes.len() > 36 {
            Err(ReadBytesBlockstampError::TooLong())
        } else if bytes.len() < 36 {
            Err(ReadBytesBlockstampError::TooShort())
        } else {
            // read id
            let mut id_bytes = Cursor::new(bytes[0..4].to_vec());
            let id = BlockId(id_bytes.read_u32::<BigEndian>()?);
            // read hash
            let mut hash_datas: [u8; 32] = [0u8; 32];
            hash_datas.copy_from_slice(&bytes[4..36]);
            let hash = BlockHash(Hash(hash_datas));
            // return Blockstamp
            Ok(Blockstamp { id, hash })
        }
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        // BlockId
        let mut buffer = [0u8; mem::size_of::<u32>()];
        buffer
            .as_mut()
            .write_u32::<BigEndian>(self.id.0)
            .expect("Unable to write");
        bytes.extend_from_slice(&buffer);
        // BlockHash
        bytes.extend(self.hash.0.to_bytes_vector());
        bytes
    }
}*/

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

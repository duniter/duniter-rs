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

//! Provide base convertion tools

use thiserror::Error;

/// Base16 conversion tools
pub mod b16;

/// Base58 conversion tools
pub mod b58;

/// Base64 conversion tools
pub mod b64;

/// Errors enumeration for Base58/64 strings convertion.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum BaseConvertionError {
    #[error("Data have invalid key length : expected {expected:?}, found {found:?}.")]
    /// Data have invalid length.
    InvalidLength {
        /// Expected length
        expected: usize,
        /// Actual length
        found: usize,
    },
    #[error("Invalid character '{character:?}' at offset {offset:?}.")]
    /// Base58/64 have an invalid character.
    InvalidCharacter {
        /// Character
        character: char,
        /// Offset (=position)
        offset: usize,
    },
    #[error("Invalid base converter length.")]
    /// Base58/64 have invalid lendth
    InvalidBaseConverterLength,
    #[error("Invalid last symbol '{symbol:?}' at offset {offset:?}.")]
    /// Base64 have invalid last symbol (symbol, offset)
    InvalidLastSymbol {
        /// Symbol
        symbol: u8,
        /// Offset (=position)
        offset: usize,
    },
    /// Unknown error
    #[error("Unknown error.")]
    UnknownError,
}

impl From<base64::DecodeError> for BaseConvertionError {
    fn from(err: base64::DecodeError) -> Self {
        match err {
            base64::DecodeError::InvalidByte(offset, byte) => {
                BaseConvertionError::InvalidCharacter {
                    character: byte as char,
                    offset,
                }
            }
            base64::DecodeError::InvalidLength => BaseConvertionError::InvalidBaseConverterLength,
            base64::DecodeError::InvalidLastSymbol(offset, symbol) => {
                BaseConvertionError::InvalidLastSymbol { symbol, offset }
            }
        }
    }
}

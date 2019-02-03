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

//! Provide base58 convertion tools

pub use base58::ToBase58;

use crate::bases::BaseConvertionError;
use base58::{FromBase58, FromBase58Error};

/// Create an array of 32 bytes from a Base58 string.
pub fn str_base58_to_32bytes(base58_data: &str) -> Result<[u8; 32], BaseConvertionError> {
    match base58_data.from_base58() {
        Ok(result) => {
            if result.len() == 32 {
                let mut u8_array = [0; 32];

                u8_array[..32].clone_from_slice(&result[..32]);

                Ok(u8_array)
            } else {
                Err(BaseConvertionError::InvalidLength {
                    expected: 32,
                    found: result.len(),
                })
            }
        }
        Err(FromBase58Error::InvalidBase58Character(character, offset)) => {
            Err(BaseConvertionError::InvalidCharacter { character, offset })
        }
        Err(FromBase58Error::InvalidBase58Length) => {
            Err(BaseConvertionError::InvalidBaseConverterLength)
        }
    }
}

/// Create an array of 64bytes from a Base58 string.
pub fn str_base58_to_64bytes(base58_data: &str) -> Result<[u8; 64], BaseConvertionError> {
    match base58_data.from_base58() {
        Ok(result) => {
            if result.len() == 64 {
                let mut u8_array = [0; 64];

                u8_array[..64].clone_from_slice(&result[..64]);

                Ok(u8_array)
            } else {
                Err(BaseConvertionError::InvalidLength {
                    expected: 64,
                    found: result.len(),
                })
            }
        }
        Err(FromBase58Error::InvalidBase58Character(character, offset)) => {
            Err(BaseConvertionError::InvalidCharacter { character, offset })
        }
        Err(FromBase58Error::InvalidBase58Length) => {
            Err(BaseConvertionError::InvalidBaseConverterLength)
        }
    }
}

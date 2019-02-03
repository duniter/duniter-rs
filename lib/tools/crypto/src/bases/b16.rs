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

//! Provide base16 convertion tools

use crate::bases::BaseConvertionError;

/// Convert a hexadecimal string in an array of 32 bytes.
///
/// The hex string must only contains hex characters
/// and produce a 32 bytes value.
pub fn str_hex_to_32bytes(text: &str) -> Result<[u8; 32], BaseConvertionError> {
    if text.len() != 64 {
        Err(BaseConvertionError::InvalidLength {
            expected: 64,
            found: text.len(),
        })
    } else {
        let mut bytes = [0u8; 32];

        let chars: Vec<char> = text.chars().collect();

        for i in 0..64 {
            if i % 2 != 0 {
                continue;
            }

            let byte1 = chars[i].to_digit(16);
            let byte2 = chars[i + 1].to_digit(16);

            if byte1.is_none() {
                return Err(BaseConvertionError::InvalidCharacter {
                    character: chars[i],
                    offset: i,
                });
            } else if byte2.is_none() {
                return Err(BaseConvertionError::InvalidCharacter {
                    character: chars[i + 1],
                    offset: i + 1,
                });
            }

            let byte1 = byte1.unwrap() as u8;
            let byte2 = byte2.unwrap() as u8;

            let byte = (byte1 << 4) | byte2;
            bytes[i / 2] = byte;
        }

        Ok(bytes)
    }
}

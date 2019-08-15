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

//! Provide base64 convertion tools

use crate::bases::BaseConvertionError;

/// Create an array of 64 bytes from a Base64 string.
pub fn str_base64_to64bytes(base64_data: &str) -> Result<[u8; 64], BaseConvertionError> {
    let result = base64::decode(base64_data)?;

    if result.len() == 64 {
        let mut u8_array = [0; 64];
        u8_array[..64].clone_from_slice(&base64::decode(base64_data)?[..64]);

        Ok(u8_array)
    } else {
        Err(BaseConvertionError::InvalidLength {
            found: result.len(),
            expected: 64,
        })
    }
}

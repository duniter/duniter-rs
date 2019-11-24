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

//! Verify block pow

use dup_crypto::hashs::Hash;
use durs_common_tools::traits::bool_ext::BoolExt;

static ZERO_STRING: &str = "0";

/// Proof of Work error
#[derive(Debug, PartialEq)]
pub enum BlockPoWError {
    /// Invalid pow_min
    _InvalidPoWMin,
    /// Invalid PoW pattern
    InvalidHashPattern {
        expected_pattern: String,
        actual_hash: String,
    },
}

pub fn verify_hash_pattern(hash: Hash, diffi: usize) -> Result<(), BlockPoWError> {
    let hash_string = hash.to_hex();
    let nb_zeros = diffi / 16;
    let expected_pattern_last_hex_digit = 16 - (diffi % 16);
    let repeated_zero_string = ZERO_STRING.repeat(nb_zeros);
    let expected_pattern = if expected_pattern_last_hex_digit < 15 && nb_zeros < 64 {
        let expected_pattern_last_char =
            std::char::from_digit(expected_pattern_last_hex_digit as u32, 16)
                .expect("expected_pattern_last_hex_digit is necessarily less than 16");
        let expected_pattern = format!(
            "{}[0-{}]*",
            repeated_zero_string, expected_pattern_last_char
        );
        let actual_pattern_last_hex_digit = usize::from_str_radix(
            hash_string
                .get(nb_zeros..=nb_zeros)
                .expect("Hash string is necessary greater than nb_zeros + 1."),
            16,
        )
        .expect("Hash type guarantees a valid hexadecimal string.");
        // remainder must be less than or equal to expected_end_pattern
        (actual_pattern_last_hex_digit <= expected_pattern_last_hex_digit).or_err(
            BlockPoWError::InvalidHashPattern {
                expected_pattern: expected_pattern.clone(),
                actual_hash: hash_string.clone(),
            },
        )?;
        expected_pattern
    } else {
        repeated_zero_string.clone()
    };
    hash_string
        .starts_with(&repeated_zero_string)
        .or_err(BlockPoWError::InvalidHashPattern {
            expected_pattern,
            actual_hash: hash_string,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_verify_hash_pattern() {
        assert_eq!(
            Ok(()),
            verify_hash_pattern(
                Hash::from_hex("000003619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                70
            )
        );
        assert_eq!(
            Ok(()),
            verify_hash_pattern(
                Hash::from_hex("000013619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                70
            )
        );
        assert_eq!(
            Err(BlockPoWError::InvalidHashPattern {
                expected_pattern: "0000[0-a]*".to_owned(),
                actual_hash: "0000B3619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559"
                    .to_owned(),
            }),
            verify_hash_pattern(
                Hash::from_hex("0000B3619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                70
            )
        );
        assert_eq!(
            Err(BlockPoWError::InvalidHashPattern {
                expected_pattern: "0000[0-a]*".to_owned(),
                actual_hash: "000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559"
                    .to_owned(),
            }),
            verify_hash_pattern(
                Hash::from_hex("000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                70
            )
        );
        assert_eq!(
            Err(BlockPoWError::InvalidHashPattern {
                expected_pattern: "00000".to_owned(),
                actual_hash: "000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559"
                    .to_owned(),
            }),
            verify_hash_pattern(
                Hash::from_hex("000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                80
            )
        );
        assert_eq!(
            Err(BlockPoWError::InvalidHashPattern {
                expected_pattern: "0000".to_owned(),
                actual_hash: "000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559"
                    .to_owned(),
            }),
            verify_hash_pattern(
                Hash::from_hex("000313619ACBF80298F074D8339175901425BC97EF528ED02EBD73CD4CA5C559")
                    .expect("invalid hash"),
                65
            )
        );
    }
}

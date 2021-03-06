//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Common rust functions to (un)escape strings.

/// Unescape backslash
pub fn unescape_str(source: &str) -> String {
    let mut previous_char = None;
    let mut str_result = String::with_capacity(source.len());

    for current_char in source.chars() {
        if let Some('\\') = previous_char {
            match current_char {
                '\\' => {} // Do nothing
                _ => str_result.push(current_char),
            }
        } else {
            str_result.push(current_char);
        }
        previous_char = Some(current_char);
    }

    str_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_unescape_double_backslash() {
        assert_eq!("\\".to_owned(), unescape_str("\\\\"));
    }

    #[test]
    pub fn test_no_unescape_single_backslash() {
        assert_eq!("ab\\cd".to_owned(), unescape_str("ab\\cd"));
    }
    #[test]
    pub fn test_unescape_str() {
        assert_eq!("abcd".to_owned(), unescape_str("abcd"));
    }
}

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

//! Common rust tools for DURS project.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate log;

/// Interrupts the program and log error message
pub fn fatal_error(msg: &str) {
    if cfg!(feature = "log_panics") {
        panic!(format!("Fatal Error : {}", msg));
    } else {
        error!("{}", &format!("Fatal Error : {}", msg));
        panic!(format!("Fatal Error : {}", msg));
    }
}

/// Unescape backslash
pub fn unescape_str(source: &str) -> String {
    let mut previous_char = None;
    let mut str_result = String::with_capacity(source.len());

    for current_char in source.chars() {
        if previous_char.is_some() && previous_char.unwrap() == '\\' {
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
    pub fn test_unescape_str() {
        assert_eq!("\\".to_owned(), unescape_str("\\\\"));
    }
}

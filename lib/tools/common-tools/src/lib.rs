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
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod fns;
pub mod macros;
pub mod traits;

/// Percent
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Percent(u8);

/// Percent error
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PercentError {
    /// Integer too large (greater than 100)
    TooLarge(u8),
}

impl Percent {
    /// New percent
    pub fn new(percent: u8) -> Result<Percent, PercentError> {
        if percent <= 100 {
            Ok(Percent(percent))
        } else {
            Err(PercentError::TooLarge(percent))
        }
    }
}

impl Into<u8> for Percent {
    fn into(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_percent() {
        assert_eq!(Percent::new(101), Err(PercentError::TooLarge(101)));

        let percent = Percent::new(100).expect("wrong percent");
        let percent_value: u8 = percent.into();
        assert_eq!(percent_value, 100u8);
    }
}

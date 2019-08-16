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

//! Implements the Duniter Documents Protocol.

use crate::CurrencyName;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};

/// CURRENCY_NULL
const CURRENCY_NULL: u16 = 0x_0000;
/// CURRENCY_G1
const CURRENCY_G1: u16 = 0x_0001;
/// CURRENCY_G1_TEST
const CURRENCY_G1_TEST: u16 = 0x_1000;

/// CurrencyCodeError
#[derive(Debug)]
pub enum CurrencyCodeError {
    /// UnknowCurrencyCode
    UnknowCurrencyCode,
    /// IoError
    IoError(std::io::Error),
    /// UnknowCurrencyName
    UnknowCurrencyName,
}

impl From<std::io::Error> for CurrencyCodeError {
    fn from(error: std::io::Error) -> Self {
        CurrencyCodeError::IoError(error)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Hash)]
/// Currency code
pub struct CurrencyCode(u16);

impl TryFrom<CurrencyName> for CurrencyCode {
    type Error = CurrencyCodeError;

    fn try_from(currency_name: CurrencyName) -> Result<Self, Self::Error> {
        match currency_name.0.as_str() {
            "g1" => Ok(CurrencyCode(CURRENCY_G1)),
            "g1-test" => Ok(CurrencyCode(CURRENCY_G1_TEST)),
            _ => Err(CurrencyCodeError::UnknowCurrencyName),
        }
    }
}

impl TryInto<CurrencyName> for CurrencyCode {
    type Error = CurrencyCodeError;

    fn try_into(self) -> Result<CurrencyName, Self::Error> {
        match self.0 {
            CURRENCY_NULL => Ok(CurrencyName("".to_owned())),
            CURRENCY_G1 => Ok(CurrencyName("g1".to_owned())),
            CURRENCY_G1_TEST => Ok(CurrencyName("g1-test".to_owned())),
            _ => Err(CurrencyCodeError::UnknowCurrencyCode),
        }
    }
}

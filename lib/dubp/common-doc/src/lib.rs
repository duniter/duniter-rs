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

//! Provide common tools for DUBP.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate pest_derive;

pub mod blockstamp;
pub mod errors;
pub mod parser;
pub mod traits;

use dup_crypto::hashs::Hash;
use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use std::fmt::{Debug, Display, Error, Formatter};

pub use blockstamp::{Blockstamp, PreviousBlockstamp};

/// Currency name
#[derive(Debug, Clone, Eq, Hash, PartialEq, Deserialize, Serialize, Shrinkwrap)]
pub struct CurrencyName(pub String);

impl Display for CurrencyName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for CurrencyName {
    fn from(s: &str) -> Self {
        CurrencyName(s.to_owned())
    }
}

/// A block number.
#[derive(
    Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Shrinkwrap,
)]
pub struct BlockNumber(pub u32);

impl Display for BlockNumber {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

/// Wrapper of a block hash.
#[derive(
    Copy, Clone, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Shrinkwrap,
)]
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

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

//! Provide wrappers around cryptographic seed

use crate::bases::*;
use base58::ToBase58;
use durs_common_tools::fatal_error;
use log::error;
use ring::rand;
use std::fmt::{self, Debug, Display, Formatter};

/// Store a 32 bytes seed used to generate keys.
#[derive(Copy, Clone, Default, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct Seed([u8; 32]);

impl AsRef<[u8]> for Seed {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ToBase58 for Seed {
    fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl Display for Seed {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for Seed {
    // Seed { DNann1L... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "Seed {{ {} }}", self)
    }
}

impl Seed {
    #[inline]
    /// Create new seed
    pub fn new(seed_bytes: [u8; 32]) -> Seed {
        Seed(seed_bytes)
    }
    #[inline]
    /// Create seed from base58 str
    pub fn from_base58(base58_str: &str) -> Result<Self, BaseConvertionError> {
        Ok(Seed(b58::str_base58_to_32bytes(base58_str)?))
    }
    #[inline]
    /// Generate random seed
    pub fn random() -> Seed {
        if let Ok(random_bytes) = rand::generate::<[u8; 32]>(&rand::SystemRandom::new()) {
            Seed(random_bytes.expose())
        } else {
            fatal_error!("System error: fail to generate random seed !")
        }
    }
}

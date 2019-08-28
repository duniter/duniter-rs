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

//! Provide wrappers around cryptographic seeds

use crate::bases::*;
use base58::ToBase58;
use durs_common_tools::fatal_error;
use log::error;
use ring::rand;
use std::fmt::{self, Debug, Display, Formatter};

/// Store a 48 bytes seed used to generate keys.
#[derive(Clone, Copy)]
pub struct Seed48([u8; 48]);

impl AsRef<[u8]> for Seed48 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Seed48 {
    fn default() -> Self {
        Seed48([0u8; 48])
    }
}

impl Seed48 {
    #[inline]
    /// Create new seed
    pub fn new(seed_bytes: [u8; 48]) -> Seed48 {
        Seed48(seed_bytes)
    }
    #[inline]
    /// Generate random seed
    pub fn random() -> Seed48 {
        if let Ok(random_bytes) = rand::generate::<[u8; 48]>(&rand::SystemRandom::new()) {
            Seed48::new(random_bytes.expose())
        } else {
            fatal_error!("System error: fail to generate random seed !")
        }
    }
}

/// Store a 32 bytes seed used to generate keys.
#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct Seed32([u8; 32]);

impl AsRef<[u8]> for Seed32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ToBase58 for Seed32 {
    fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl Display for Seed32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for Seed32 {
    // Seed32 { DNann1L... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "Seed32 {{ {} }}", self)
    }
}

impl Seed32 {
    #[inline]
    /// Create new seed
    pub fn new(seed_bytes: [u8; 32]) -> Seed32 {
        Seed32(seed_bytes)
    }
    #[inline]
    /// Create seed from base58 str
    pub fn from_base58(base58_str: &str) -> Result<Self, BaseConvertionError> {
        Ok(Seed32::new(b58::str_base58_to_32bytes(base58_str)?))
    }
    #[inline]
    /// Generate random seed
    pub fn random() -> Seed32 {
        if let Ok(random_bytes) = rand::generate::<[u8; 32]>(&rand::SystemRandom::new()) {
            Seed32::new(random_bytes.expose())
        } else {
            fatal_error!("System error: fail to generate random seed !")
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_gen_random_seed() {
        assert_ne!(Seed32::random(), Seed32::random());
    }
}

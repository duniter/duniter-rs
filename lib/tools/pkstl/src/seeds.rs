//  Copyright (C) 2017-2019  Eloïs SANCHEZ.
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

use clear_on_drop::clear::Clear;

/// Store a 32 bytes seed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seed32([u8; 32]);

impl AsRef<[u8]> for Seed32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Seed32 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Drop for Seed32 {
    #[inline]
    fn drop(&mut self) {
        <[u8; 32] as Clear>::clear(&mut self.0);
    }
}

impl Seed32 {
    #[inline]
    /// Create new seed
    pub fn new(seed_bytes: [u8; 32]) -> Seed32 {
        Seed32(seed_bytes)
    }
    #[inline]
    /// Generate random seed
    pub fn random() -> Seed32 {
        if let Ok(random_bytes) = ring::rand::generate::<[u8; 32]>(&ring::rand::SystemRandom::new())
        {
            Seed32::new(random_bytes.expose())
        } else {
            panic!("System error: fail to generate random seed !")
        }
    }
}

/// Store a 48 bytes seed.
#[derive(Default)]
pub struct Seed48(InnerSeed48);

struct InnerSeed48([u8; 48]);

impl Default for InnerSeed48 {
    fn default() -> Self {
        InnerSeed48([0u8; 48])
    }
}

impl AsRef<[u8]> for Seed48 {
    fn as_ref(&self) -> &[u8] {
        &(self.0).0
    }
}

impl AsMut<[u8]> for Seed48 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut (self.0).0
    }
}

impl Drop for Seed48 {
    #[inline]
    fn drop(&mut self) {
        <InnerSeed48 as Clear>::clear(&mut self.0);
    }
}

#[cfg(test)]
impl Seed48 {
    #[inline]
    /// Create new seed
    pub fn new(seed_bytes: [u8; 48]) -> Seed48 {
        Seed48(InnerSeed48(seed_bytes))
    }
}

/// Store a 64 bytes seed.
#[derive(Default)]
pub struct Seed64(InnerSeed64);

struct InnerSeed64([u8; 64]);

impl Default for InnerSeed64 {
    fn default() -> Self {
        InnerSeed64([0u8; 64])
    }
}

impl AsMut<[u8]> for Seed64 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut (self.0).0
    }
}

impl Drop for Seed64 {
    #[inline]
    fn drop(&mut self) {
        <InnerSeed64 as Clear>::clear(&mut self.0);
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[inline]
    /// Generate random Seed48
    pub fn random_seed_48() -> Seed48 {
        if let Ok(random_bytes) = ring::rand::generate::<[u8; 48]>(&ring::rand::SystemRandom::new())
        {
            Seed48::new(random_bytes.expose())
        } else {
            panic!("System error: fail to generate random seed !")
        }
    }

    #[test]
    fn tests_seed32() {
        assert_ne!(Seed32::random(), Seed32::random());

        let mut seed = Seed32::new([3u8; 32]);

        assert_eq!(&[3u8; 32], seed.as_ref());
        assert_eq!(&mut [3u8; 32], seed.as_mut());
    }
}

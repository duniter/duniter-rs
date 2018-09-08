//  Copyright (C) 2018  The Duniter Project Developers.
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

#[derive(Clone, Debug, Eq, PartialEq)]
/// WS2PFeatures
pub struct WS2PFeatures(pub Vec<u8>);

impl WS2PFeatures {
    /// Return true if all flags are disabled (or if it's really empty).
    pub fn is_empty(&self) -> bool {
        for byte in &self.0 {
            if *byte > 0u8 {
                return false;
            }
        }
        true
    }
    /// Check flag DEF
    pub fn _def(&self) -> bool {
        self.0[0] | 0b1111_1110 == 255u8
    }
    /// Check flag LOW
    pub fn _low(&self) -> bool {
        self.0[0] | 0b1111_1101 == 255u8
    }
    /// Check flag ABF
    pub fn _abf(&self) -> bool {
        self.0[0] | 0b1111_1011 == 255u8
    }
}

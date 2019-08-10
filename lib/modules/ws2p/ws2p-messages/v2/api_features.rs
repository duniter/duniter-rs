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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub fn def(&self) -> bool {
        self.0[0] | 0b1111_1110 == 255u8
    }
    /// Check flag LOW
    pub fn low(&self) -> bool {
        self.0[0] | 0b1111_1101 == 255u8
    }
    /// Check flag ABF
    pub fn abf(&self) -> bool {
        self.0[0] | 0b1111_1011 == 255u8
    }
    /// Check features compatibility
    pub fn check_features_compatibility(
        &self,
        remote_features: &WS2PFeatures,
    ) -> Result<WS2PFeatures, ()> {
        let mut merged_features = self.clone();
        // Remove features unsuported by remote node
        if self.def() && !remote_features.def() {
            merged_features.0[0] &= 0b1111_1110;
        }
        if self.low() && !remote_features.low() {
            merged_features.0[0] &= 0b1111_1101;
        }
        if self.abf() && !remote_features.abf() {
            merged_features.0[0] &= 0b1111_1011;
        }
        // Check incompatiblities
        if remote_features.low() && !self.low() {
            Err(())
        } else {
            Ok(merged_features)
        }
    }
}

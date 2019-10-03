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

//! Common rust functions for extend u64 capabilities.

/// Convert an u64 to two u32
pub fn from_2_u32(a: u32, b: u32) -> u64 {
    let a_bytes = a.to_be_bytes();
    let b_bytes = b.to_be_bytes();
    let mut u64_bytes = [0u8; 8];
    u64_bytes[..4].copy_from_slice(&a_bytes[..4]);
    u64_bytes[4..8].copy_from_slice(&b_bytes[..4]);
    u64::from_be_bytes(u64_bytes)
}

/// Create an u64 from two u32
pub fn to_2_u32(u64_: u64) -> (u32, u32) {
    let mut a = [0u8; 4];
    let mut b = [0u8; 4];
    let u64_bytes = u64_.to_be_bytes();
    a.copy_from_slice(&u64_bytes[..4]);
    b.copy_from_slice(&u64_bytes[4..]);

    (u32::from_be_bytes(a), u32::from_be_bytes(b))
}

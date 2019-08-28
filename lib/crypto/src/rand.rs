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

//! Manage random generation.

use byteorder::ByteOrder;
use durs_common_tools::fatal_error;
use log::error;
use ring::rand;

#[inline]
/// Generate random u32
pub fn gen_u32() -> u32 {
    let rng = rand::SystemRandom::new();
    if let Ok(random_bytes) = rand::generate::<[u8; 4]>(&rng) {
        byteorder::BigEndian::read_u32(&random_bytes.expose())
    } else {
        fatal_error!("System error: fail to generate random boolean !")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_gen_u32() {
        assert_ne!(gen_u32(), gen_u32())
    }
}

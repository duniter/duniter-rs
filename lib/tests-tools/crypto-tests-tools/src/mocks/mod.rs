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

//! Crypto mocks for projects use dup-crypto

use dup_crypto::hashs::Hash;

/// Generate mock hash from one character
pub fn hash(character: char) -> Hash {
    let str_hash: String = (0..64).into_iter().map(|_| character).collect();

    Hash::from_hex(&str_hash).expect("Fail to create hash !")
}

/// Generate mock hash from one byte
pub fn hash_from_byte(byte: u8) -> Hash {
    let mut hash_bin = [0u8; 32];
    for b in &mut hash_bin {
        *b = byte
    }

    Hash(hash_bin)
}

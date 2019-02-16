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

//! Crypto mocks for projects use dubp-documents

use dubp_documents::BlockHash;
use dubp_documents::{BlockId, Blockstamp};

/// Generate n mock blockstamps
pub fn generate_blockstamps(n: usize) -> Vec<Blockstamp> {
    (0..n)
        .into_iter()
        .map(|i| Blockstamp {
            id: BlockId(i as u32),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash_from_byte(
                (i % 255) as u8,
            )),
        })
        .collect()
}

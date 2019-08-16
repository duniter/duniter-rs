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

//! Verify block inner hash and block hash

use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait, VerifyBlockHashError};

/// Verify block hashs
pub fn verify_block_hashs(block_doc: &BlockDocument) -> Result<(), VerifyBlockHashError> {
    trace!("complete_block #{}...", block_doc.number());

    match block_doc.verify_inner_hash() {
        Ok(()) => block_doc.verify_hash().map_err(|e| {
            warn!("BlockchainModule : Refuse Bloc : invalid hash !");
            e
        }),
        Err(e) => {
            warn!("BlockchainModule : Refuse Bloc : invalid inner hash !");
            warn!("BlockDocument=\"{:?}\"", block_doc);
            warn!(
                "BlockInnerFormat=\"{}\"",
                block_doc.generate_compact_inner_text()
            );
            Err(e)
        }
    }
}

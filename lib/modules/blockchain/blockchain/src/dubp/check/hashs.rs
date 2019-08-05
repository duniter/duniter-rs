//  Copyright (C) 2018  The Dunitrust Project Developers.
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

use crate::VerifyBlockHashsError;
use dubp_documents::documents::block::{BlockDocument, BlockDocumentTrait};

/// Verify block hashs
pub fn verify_block_hashs(block_doc: &BlockDocument) -> Result<(), VerifyBlockHashsError> {
    trace!("complete_block #{}...", block_doc.number());

    if block_doc.verify_inner_hash() {
        if block_doc.verify_hash() {
            trace!("Succes to verify_block_hashs #{}", block_doc.number().0);
            Ok(())
        } else {
            warn!("BlockchainModule : Refuse Bloc : invalid hash !");
            Err(VerifyBlockHashsError::InvalidHash(
                block_doc.number(),
                block_doc.hash(),
            ))
        }
    } else {
        warn!("BlockchainModule : Refuse Bloc : invalid inner hash !");
        warn!("BlockDocument=\"{:?}\"", block_doc);
        warn!(
            "BlockInnerFormat=\"{}\"",
            block_doc.generate_compact_inner_text()
        );
        Err(VerifyBlockHashsError::InvalidInnerHash())
    }
}

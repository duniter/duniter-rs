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

//! Sub-module checking if a block complies with all the rules of the (DUBP DUniter Blockchain Protocol).

pub mod global;
pub mod hashs;
pub mod local;
pub mod pow;

use crate::dubp::BlockError;
use crate::BlockchainModule;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockNumber, Blockstamp};
use durs_bc_db_reader::BcDbInReadTx;
use durs_common_tools::traits::bool_ext::BoolExt;
use unwrap::unwrap;

#[derive(Debug)]
pub enum CheckBlockError {
    Global(global::GlobalVerifyBlockError),
    Hashs(dubp_block_doc::block::VerifyBlockHashError),
    Local(local::LocalVerifyBlockError),
    Pow(pow::InvalidHashPattern),
}

#[derive(Debug)]
pub enum BlockChainability {
    FullyValidAndChainableBLock,
    LocalValidAndUnchainableBlock,
}

/// Check block validity
pub fn check_block<DB: BcDbInReadTx>(
    bc: &mut BlockchainModule,
    db: &DB,
    block_doc: &BlockDocument,
) -> Result<BlockChainability, BlockError> {
    let already_have_block;
    if bc.cautious_mode {
        // Check if we already have the block
        // VERY IMPORTANT: there are cases where it's legitimate to check a block that we already have.
        // For example, in case of rollback or application of orphan blocks that have become stackable.
        // The fact of having already received the block is a cause of rejection only if the block is not chainable.
        already_have_block = durs_bc_db_reader::blocks::already_have_block(
            db,
            block_doc.blockstamp(),
            block_doc.previous_hash(),
        )
        .map_err(BlockError::DbError)?;

        if already_have_block.not() {
            // Verify proof of work
            // The case where the block has none hash is captured by check_block_hashes below
            if let Some(hash) = block_doc.hash() {
                pow::verify_hash_pattern(hash.0, block_doc.pow_min().into())
                    .map_err(CheckBlockError::Pow)?;
            }
            // Check block hashes.
            crate::dubp::check::hashs::check_block_hashes(block_doc)
                .map_err(CheckBlockError::Hashs)?;
        }
    } else {
        // If we're not in cautious mode, we still need to check the block hashes.
        crate::dubp::check::hashs::check_block_hashes(block_doc).map_err(CheckBlockError::Hashs)?;
        already_have_block = false;
    };

    // Check block chainability
    if (block_doc.number().0 == 0 && bc.current_blockstamp == Blockstamp::default())
        || (block_doc.number().0 == bc.current_blockstamp.id.0 + 1
            && unwrap!(block_doc.previous_hash()).to_string()
                == bc.current_blockstamp.hash.0.to_string())
    {
        if bc.cautious_mode {
            debug!("check_block: block {} chainable!", block_doc.blockstamp());

            // Local verification
            local::verify_local_validity_block(block_doc, bc.currency_params)
                .map_err(CheckBlockError::Local)?;

            // Verify block validity (check all protocol rule, very long !)
            if block_doc.number() > BlockNumber(0) {
                global::verify_global_validity_block(
                    block_doc,
                    db,
                    &bc.wot_index,
                    &bc.wot_databases.wot_db,
                )
                .map_err(CheckBlockError::Global)?;
            }

            debug!(
                "check_block: block {} is fully valid.!",
                block_doc.blockstamp()
            );
        }

        Ok(BlockChainability::FullyValidAndChainableBLock)
    } else {
        // Check that we don't already have the block
        already_have_block
            .not()
            .or_err(BlockError::AlreadyHaveBlock)?;

        // TODO check estimate pow_min

        Ok(BlockChainability::LocalValidAndUnchainableBlock)
    }
}

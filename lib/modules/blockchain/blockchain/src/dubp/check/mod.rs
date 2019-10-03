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

pub mod hashs;

use crate::dubp::BlockError;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::PubKey;
use durs_bc_db_reader::DbReader;
use durs_bc_db_writer::*;
use durs_wot::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum InvalidBlockError {
    _InvalidPow {
        expected_pattern: String,
        block_hash: Hash,
    },
    NoPreviousBlock,
    VersionDecrease,
}

pub fn verify_block_validity<DB, R, W>(
    block: &BlockDocument,
    db: &DB,
    r: &R,
    _wot_index: &HashMap<PubKey, WotId>,
    _wot_db: &BinFreeStructDb<W>,
) -> Result<(), BlockError>
where
    DB: DbReadable,
    R: DbReader,
    W: WebOfTrust,
{
    // Rules that do not concern genesis block
    if block.number().0 > 0 {
        // Get previous block
        let previous_block_opt = durs_bc_db_reader::blocks::get_block_in_local_blockchain(
            db,
            r,
            BlockNumber(block.number().0 - 1),
        )?;

        // Previous block must exist
        if previous_block_opt.is_none() {
            return Err(BlockError::InvalidBlock(InvalidBlockError::NoPreviousBlock));
        }
        let previous_block = previous_block_opt.expect("safe unwrap");

        // Block version must not decrease
        if previous_block.version() > block.version() {
            return Err(BlockError::InvalidBlock(InvalidBlockError::VersionDecrease));
        }
    }

    Ok(())
}

fn _verify_pow<DB: DbReadable, R: DbReader>(
    _db: &DB,
    _r: &R,
    _wot_id: WotId,
    _block_hash: Hash,
    _block_pow_min: u32,
) -> Result<(), InvalidBlockError> {
    unimplemented!();
}

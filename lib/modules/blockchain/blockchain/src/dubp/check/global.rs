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

use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dup_crypto::keys::PubKey;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
use durs_bc_db_writer::BinFreeStructDb;
use durs_common_tools::traits::bool_ext::BoolExt;
use durs_wot::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum GlobalVerifyBlockError {
    DbError(DbError),
    NoPreviousBlock,
    VersionDecrease,
}

impl From<DbError> for GlobalVerifyBlockError {
    fn from(err: DbError) -> Self {
        GlobalVerifyBlockError::DbError(err)
    }
}

pub fn verify_global_validity_block<DB, W>(
    block: &BlockDocument,
    db: &DB,
    _wot_index: &HashMap<PubKey, WotId>,
    _wot_db: &BinFreeStructDb<W>,
) -> Result<(), GlobalVerifyBlockError>
where
    DB: BcDbInReadTx,
    W: WebOfTrust,
{
    // Rules that do not concern genesis block
    if block.number().0 > 0 {
        // Get previous block
        let previous_block_opt = durs_bc_db_reader::blocks::get_block_in_local_blockchain(
            db,
            BlockNumber(block.number().0 - 1),
        )?;

        // Previous block must exist
        previous_block_opt
            .is_some()
            .or_err(GlobalVerifyBlockError::NoPreviousBlock)?;
        let previous_block = previous_block_opt.expect("safe unwrap");

        // Block version must not decrease
        (block.version() >= previous_block.version())
            .or_err(GlobalVerifyBlockError::VersionDecrease)?;
    }

    // TODO BR_G100 - issuerIsMember
    Ok(())
}

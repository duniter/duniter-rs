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

//! Sub-module that checks and applies the content of a block according to the DUBP (DUBP DUniter Blockchain Protocol).

pub mod apply;
pub mod check;

use crate::*;
use apply::*;
use check::*;
use dubp_block_doc::block::{BlockDocumentTrait, VerifyBlockHashError};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockNumber, Blockstamp};
use durs_blockchain_dal::entities::block::DALBlock;
use durs_blockchain_dal::*;
use unwrap::unwrap;

#[derive(Debug, Clone)]
pub enum CheckAndApplyBlockReturn {
    ValidMainBlock(ValidBlockApplyReqs),
    ForkBlock,
    OrphanBlock,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockError {
    AlreadyHaveBlock,
    BlockOrOutForkWindow,
    VerifyBlockHashError(VerifyBlockHashError),
    DALError(DALError),
    InvalidBlock(InvalidBlockError),
    ApplyValidBlockError(ApplyValidBlockError),
}

impl From<VerifyBlockHashError> for BlockError {
    fn from(err: VerifyBlockHashError) -> Self {
        BlockError::VerifyBlockHashError(err)
    }
}

impl From<DALError> for BlockError {
    fn from(err: DALError) -> Self {
        BlockError::DALError(err)
    }
}

impl From<ApplyValidBlockError> for BlockError {
    fn from(err: ApplyValidBlockError) -> Self {
        BlockError::ApplyValidBlockError(err)
    }
}

pub fn check_and_apply_block(
    bc: &mut BlockchainModule,
    block_doc: BlockDocument,
) -> Result<CheckAndApplyBlockReturn, BlockError> {
    // Get BlockDocument && check if already have block
    let already_have_block = readers::block::already_have_block(
        &bc.blocks_databases.blockchain_db,
        &bc.forks_dbs,
        block_doc.blockstamp(),
        block_doc.previous_hash(),
    )?;

    // Verify block hashs
    dubp::check::hashs::verify_block_hashs(&block_doc)?;

    // Check block chainability
    if (block_doc.number().0 == 0 && bc.current_blockstamp == Blockstamp::default())
        || (block_doc.number().0 == bc.current_blockstamp.id.0 + 1
            && unwrap!(block_doc.previous_hash()).to_string()
                == bc.current_blockstamp.hash.0.to_string())
    {
        debug!(
            "stackable_block : block {} chainable !",
            block_doc.blockstamp()
        );
        // Detect expire_certs
        let blocks_expiring = Vec::with_capacity(0);
        let expire_certs = durs_blockchain_dal::readers::certs::find_expire_certs(
            &bc.wot_databases.certs_db,
            blocks_expiring,
        )?;

        // Verify block validity (check all protocol rule, very long !)
        verify_block_validity(
            &block_doc,
            &bc.blocks_databases.blockchain_db,
            &bc.wot_databases.certs_db,
            &bc.wot_index,
            &bc.wot_databases.wot_db,
        )?;

        // If we're in block genesis, get the currency parameters
        if block_doc.number() == BlockNumber(0) {
            // Open currency_params_db
            let datas_path = durs_conf::get_datas_path(bc.profile_path.clone());
            // Get and write currency params
            bc.currency_params = Some(
                durs_blockchain_dal::readers::currency_params::get_and_write_currency_params(
                    &datas_path,
                    &block_doc,
                ),
            );
        }

        Ok(CheckAndApplyBlockReturn::ValidMainBlock(apply_valid_block(
            block_doc,
            &mut bc.wot_index,
            &bc.wot_databases.wot_db,
            &expire_certs,
        )?))
    } else if already_have_block {
        Err(BlockError::AlreadyHaveBlock)
    } else if block_doc.number().0 >= bc.current_blockstamp.id.0
        || (bc.current_blockstamp.id.0 - block_doc.number().0)
            < unwrap!(bc.currency_params).fork_window_size as u32
    {
        debug!(
            "stackable_block : block {} not chainable, store this for future !",
            block_doc.blockstamp()
        );

        let dal_block = DALBlock {
            block: block_doc.clone(),
            expire_certs: None,
        };

        if durs_blockchain_dal::writers::block::insert_new_fork_block(&bc.forks_dbs, dal_block)
            .expect("durs_blockchain_dal::writers::block::insert_new_fork_block() : DALError")
        {
            Ok(CheckAndApplyBlockReturn::ForkBlock)
        } else {
            Ok(CheckAndApplyBlockReturn::OrphanBlock)
        }
    } else {
        debug!(
            "stackable_block : block {} not chainable and already stored or out of forkWindowSize !",
            block_doc.blockstamp()
        );
        Err(BlockError::BlockOrOutForkWindow)
    }
}

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

use crate::dubp::apply::{ApplyValidBlockError, WriteBlockQueries};
use crate::dubp::check::CheckBlockError;
use crate::BlockchainModule;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_reader::DbError;
use durs_bc_db_writer::{BcDbRwWithWriter, Db, DbWriter};
use unwrap::unwrap;

#[derive(Debug, Clone)]
pub enum CheckAndApplyBlockReturn {
    ValidMainBlock(WriteBlockQueries),
    ForkBlock,
    OrphanBlock,
}

#[derive(Debug)]
pub enum BlockError {
    AlreadyHaveBlock,
    ApplyValidBlockError(ApplyValidBlockError),
    BlockOrOutForkWindow,
    DbError(DbError),
    InvalidBlock(CheckBlockError),
}

impl From<ApplyValidBlockError> for BlockError {
    fn from(e: ApplyValidBlockError) -> Self {
        Self::ApplyValidBlockError(e)
    }
}

impl From<DbError> for BlockError {
    fn from(err: DbError) -> Self {
        BlockError::DbError(err)
    }
}

impl From<CheckBlockError> for BlockError {
    fn from(e: CheckBlockError) -> Self {
        Self::InvalidBlock(e)
    }
}

pub fn check_and_apply_block(
    bc: &mut BlockchainModule,
    db: &Db,
    w: &mut DbWriter,
    block_doc: BlockDocument,
) -> Result<CheckAndApplyBlockReturn, BlockError> {
    match check::check_block(bc, &BcDbRwWithWriter { db, w }, &block_doc)? {
        check::BlockChainability::FullyValidAndChainableBLock => {
            treat_chainable_block(bc, db, w, block_doc)
        }
        check::BlockChainability::LocalValidAndUnchainableBlock => {
            treat_unchainable_block(bc, db, w, block_doc)
        }
    }
}

fn treat_chainable_block(
    bc: &mut BlockchainModule,
    db: &Db,
    w: &mut DbWriter,
    block_doc: BlockDocument,
) -> Result<CheckAndApplyBlockReturn, BlockError> {
    // Detect expire_certs
    let blocks_expiring = Vec::with_capacity(0); // TODO
    let expire_certs = durs_bc_db_reader::indexes::certs::find_expire_certs(
        &BcDbRwWithWriter { db, w },
        &blocks_expiring,
    )?;

    // If we're in block genesis, get the currency parameters
    if block_doc.number() == BlockNumber(0) {
        // Open currency_params_db
        let datas_path = durs_conf::get_datas_path(bc.profile_path.clone());
        // Get and write currency params
        bc.currency_params = Some(
            durs_bc_db_reader::currency_params::get_and_write_currency_params(
                &datas_path,
                &block_doc,
            ),
        );
    }

    let write_block_queries: WriteBlockQueries = crate::dubp::apply::apply_valid_block(
        db,
        w,
        block_doc,
        &mut bc.wot_index,
        &bc.wot_databases.wot_db,
        &expire_certs,
    )?;

    Ok(CheckAndApplyBlockReturn::ValidMainBlock(
        write_block_queries,
    ))
}

fn treat_unchainable_block(
    bc: &mut BlockchainModule,
    db: &Db,
    w: &mut DbWriter,
    block_doc: BlockDocument,
) -> Result<CheckAndApplyBlockReturn, BlockError> {
    if block_doc.number().0 >= bc.current_blockstamp.id.0
        || (bc.current_blockstamp.id.0 - block_doc.number().0)
            < unwrap!(bc.currency_params).fork_window_size as u32
    {
        debug!(
            "stackable_block : block {} not chainable, store this for future !",
            block_doc.blockstamp()
        );

        let dal_block = BlockDb {
            block: block_doc,
            expire_certs: None,
        };

        if durs_bc_db_writer::blocks::insert_new_fork_block(&db, w, &mut bc.fork_tree, dal_block)
            .expect("durs_bc_db_writer::writers::block::insert_new_fork_block() : DbError")
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

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

//! Sub-module that finds and applies the orphaned blocks that have become stackable on the local blockchain.

use crate::dubp::apply::exec_currency_queries;
use crate::*;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::BcDbRead;
use unwrap::unwrap;

pub fn apply_stackable_blocks(bc: &mut BlockchainModule) {
    'blocks: loop {
        let stackable_blocks =
            bc.db()
                .r(|db_r| {
                    durs_bc_db_reader::blocks::get_stackables_blocks(db_r, bc.current_blockstamp)
                })
                .expect("Fatal error : Fail to read ForksDB !");

        if stackable_blocks.is_empty() {
            break 'blocks;
        }

        for stackable_block in stackable_blocks {
            debug!("stackable_block({})", stackable_block.block.number());

            let stackable_block_number = stackable_block.block.number();
            let stackable_block_blockstamp = stackable_block.block.blockstamp();

            // Apply db requests
            let db = bc.take_db();
            let db_write_result = db.write(|mut w| {
                match check_and_apply_block(bc, &db, &mut w, stackable_block.block) {
                    Ok(CheckAndApplyBlockReturn::ValidMainBlock(WriteBlockQueries(
                        bc_db_query,
                        wot_dbs_queries,
                        tx_dbs_queries,
                    ))) => {
                        let new_current_block = bc_db_query.get_block_doc_copy();
                        let blockstamp = new_current_block.blockstamp();

                        bc_db_query
                            .apply(
                                &db,
                                &mut w,
                                &mut bc.fork_tree,
                                unwrap!(bc.currency_params).fork_window_size,
                                None,
                            )
                            .expect("DB error : Fail to apply block query !");
                        for query in &wot_dbs_queries {
                            query
                                .apply(&db, &mut w, &blockstamp, &unwrap!(bc.currency_params))
                                .expect("DB error : Fail to apply wot queries !");
                        }
                        exec_currency_queries(&db, &mut w, blockstamp.id, tx_dbs_queries)
                            .expect("DB error : Fail to apply currency queries !");
                        durs_bc_db_writer::blocks::fork_tree::save_fork_tree(
                            &db,
                            &mut w,
                            &bc.fork_tree,
                        )
                        .expect("DB error : Fail to save fork tree !");
                        debug!("success to stackable_block({})", stackable_block_number);

                        events::sent::send_event(
                            bc,
                            &BlockchainEvent::StackUpValidBlock(Box::new(new_current_block)),
                        );
                        Ok(WriteResp::new(w, stackable_block_blockstamp))
                    }
                    Ok(re) => {
                        warn!(
                            "fail to stackable_block({}) : {:?}",
                            stackable_block_number, re
                        );
                        Err(DbError::WriteAbort {
                            reason: format!("{:?}", re),
                        })
                    }
                    Err(e) => {
                        warn!(
                            "fail to stackable_block({}) : {:?}",
                            stackable_block_number, e
                        );
                        Err(DbError::WriteAbort {
                            reason: format!("{:?}", e),
                        })
                    }
                }
            });
            bc.db = Some(db);

            match db_write_result {
                Ok(new_current_blockstamp) => {
                    bc.current_blockstamp = new_current_blockstamp;
                    continue 'blocks;
                }
                Err(e) => {
                    debug!(
                        "Invalid stackable block {}: {:?}",
                        stackable_block_blockstamp, e
                    );
                }
            }
        }

        // If we reach this point, it is that none of the stackable blocks are valid
        break 'blocks;
    }
    // Save database
    bc.db()
        .save()
        .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
    bc.wot_databases.save_dbs();
}

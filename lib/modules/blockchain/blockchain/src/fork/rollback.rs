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

use crate::fork::revert_block::ValidBlockRevertReqs;
use crate::*;
use dubp_common_doc::traits::Document;
use dubp_common_doc::Blockstamp;
use durs_common_tools::fatal_error;
use unwrap::unwrap;

pub fn apply_rollback(bc: &mut BlockchainModule, new_bc_branch: Vec<Blockstamp>) {
    if new_bc_branch.is_empty() {
        return;
    }

    let old_current_blockstamp = bc.current_blockstamp;
    let last_common_block_number = new_bc_branch[0].id.0 - 1;

    // Rollback (revert old branch)
    while bc.current_blockstamp.id.0 > last_common_block_number {
        if let Some(dal_block) =
            durs_bc_db_reader::readers::block::get_fork_block(&bc.db, bc.current_blockstamp)
                .unwrap_or_else(|_| {
                    fatal_error!("revert block {} fail !", bc.current_blockstamp);
                })
        {
            let blockstamp = dal_block.block.blockstamp();
            debug!("try to revert block #{}", blockstamp);
            let ValidBlockRevertReqs {
                new_current_blockstamp,
                block_query,
                wot_queries,
                currency_queries,
            } = super::revert_block::revert_block(
                dal_block,
                &mut bc.wot_index,
                &bc.wot_databases.wot_db,
                &bc.currency_databases.tx_db,
            )
            .unwrap_or_else(|_| {
                fatal_error!("revert block {} fail !", bc.current_blockstamp);
            });
            // Update current blockstamp
            bc.current_blockstamp = new_current_blockstamp;
            // Apply db requests
            block_query
                .apply(
                    &bc.db,
                    &mut bc.fork_tree,
                    unwrap!(bc.currency_params).fork_window_size,
                    None,
                )
                .expect("Fatal error : Fail to apply DBWriteRequest !");
            for query in &wot_queries {
                query
                    .apply(
                        &blockstamp,
                        &unwrap!(bc.currency_params),
                        &bc.wot_databases,
                        &bc.db,
                    )
                    .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
            }
            for query in &currency_queries {
                query
                    .apply(&blockstamp, &bc.currency_databases)
                    .expect("Fatal error : Fail to apply CurrencyDBsWriteRequest !");
            }
            debug!("Successfully revert block #{}", blockstamp);
        } else {
            fatal_error!("apply_rollback(): Not found current block in forks blocks DB !");
        }
    }

    // Apply new branch
    let mut new_branch_is_valid = true;
    let mut new_branch_blocks = Vec::with_capacity(new_bc_branch.len());
    for blockstamp in &new_bc_branch {
        if let Ok(Some(dal_block)) =
            durs_bc_db_reader::readers::block::get_fork_block(&bc.db, *blockstamp)
        {
            new_branch_blocks.push(dal_block.clone());
            if let Ok(CheckAndApplyBlockReturn::ValidMainBlock(ValidBlockApplyReqs(
                bc_db_query,
                wot_dbs_queries,
                tx_dbs_queries,
            ))) = check_and_apply_block(bc, dal_block.block)
            {
                bc.current_blockstamp = *blockstamp;
                // Apply db requests
                bc_db_query
                    .apply(
                        &bc.db,
                        &mut bc.fork_tree,
                        unwrap!(bc.currency_params).fork_window_size,
                        None,
                    )
                    .expect("Fatal error : Fail to apply DBWriteRequest !");
                for query in &wot_dbs_queries {
                    query
                        .apply(
                            &blockstamp,
                            &unwrap!(bc.currency_params),
                            &bc.wot_databases,
                            &bc.db,
                        )
                        .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
                }
                for query in &tx_dbs_queries {
                    query
                        .apply(&blockstamp, &bc.currency_databases)
                        .expect("Fatal error : Fail to apply CurrencyDBsWriteRequest !");
                }
            } else {
                new_branch_is_valid = false;
                bc.invalid_forks.insert(*blockstamp);
                break;
            }
        } else {
            fatal_error!(
                "apply_rollback(): Fail to get block {} on new branch in forks blocks DB !",
                blockstamp
            );
        }
    }

    if new_branch_is_valid {
        // update main branch in fork tree
        if let Err(err) = durs_bc_db_writer::writers::fork_tree::change_main_branch(
            &bc.db,
            &mut bc.fork_tree,
            old_current_blockstamp,
            bc.current_blockstamp,
        ) {
            fatal_error!("DbError: ForksDB: {:?}", err);
        }

        // save dbs
        bc.db
            .save()
            .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
        durs_bc_db_writer::writers::fork_tree::save_fork_tree(&bc.db, &bc.fork_tree)
            .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
        bc.wot_databases.save_dbs();
        bc.currency_databases.save_dbs(true, true);
        // Send events stackUpValidBlock
        for db_block in new_branch_blocks {
            events::sent::send_event(
                bc,
                &BlockchainEvent::StackUpValidBlock(Box::new(db_block.block)),
            )
        }
    } else {
        // reload dbs
        let dbs_path = durs_conf::get_blockchain_db_path(bc.profile_path.clone());
        bc.wot_databases = WotsV10DBs::open(Some(&dbs_path));
        bc.currency_databases = CurrencyV10DBs::open(Some(&dbs_path));
    }
}

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

use crate::*;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use unwrap::unwrap;

pub fn apply_stackable_blocks(bc: &mut BlockchainModule) {
    'blockchain: loop {
        let stackable_blocks =
            durs_bc_db_reader::blocks::get_stackables_blocks(&bc.db, bc.current_blockstamp)
                .expect("Fatal error : Fail to read ForksDB !");
        if stackable_blocks.is_empty() {
            break 'blockchain;
        } else {
            for stackable_block in stackable_blocks {
                debug!("stackable_block({})", stackable_block.block.number());

                let stackable_block_number = stackable_block.block.number();
                let stackable_block_blockstamp = stackable_block.block.blockstamp();

                match check_and_apply_block(bc, stackable_block.block) {
                    Ok(CheckAndApplyBlockReturn::ValidMainBlock(ValidBlockApplyReqs(
                        bc_db_query,
                        wot_dbs_queries,
                        tx_dbs_queries,
                    ))) => {
                        let new_current_block = bc_db_query.get_block_doc_copy();
                        let blockstamp = new_current_block.blockstamp();
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
                        debug!("success to stackable_block({})", stackable_block_number);

                        bc.current_blockstamp = stackable_block_blockstamp;
                        events::sent::send_event(
                            bc,
                            &BlockchainEvent::StackUpValidBlock(Box::new(new_current_block)),
                        );
                        continue 'blockchain;
                    }
                    Ok(re) => warn!(
                        "fail to stackable_block({}) : {:?}",
                        stackable_block_number, re
                    ),
                    Err(e) => warn!(
                        "fail to stackable_block({}) : {:?}",
                        stackable_block_number, e
                    ),
                }
            }
            // Save databases
            bc.db
                .save()
                .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
            durs_bc_db_writer::writers::fork_tree::save_fork_tree(&bc.db, &bc.fork_tree)
                .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
            bc.wot_databases.save_dbs();
            bc.currency_databases.save_dbs(true, true);
            break 'blockchain;
        }
    }
}

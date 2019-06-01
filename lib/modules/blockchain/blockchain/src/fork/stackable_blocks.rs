//  Copyright (C) 2018  The Durs Project Developers.
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

pub fn apply_stackable_blocks(bc: &mut BlockchainModule) {
    'blockchain: loop {
        let stackable_blocks = durs_blockchain_dal::readers::fork_tree::get_stackables_blocks(
            &bc.forks_dbs,
            &bc.current_blockstamp,
        )
        .expect("Fatal error : Fail to read ForksDB !");
        if stackable_blocks.is_empty() {
            break 'blockchain;
        } else {
            for stackable_block in stackable_blocks {
                debug!("stackable_block({})", stackable_block.block.number);

                let stackable_block_number = stackable_block.block.number;
                let stackable_block_blockstamp = stackable_block.block.blockstamp();

                if let Ok(CheckAndApplyBlockReturn::ValidBlock(ValidBlockApplyReqs(
                    bc_db_query,
                    wot_dbs_queries,
                    tx_dbs_queries,
                ))) = check_and_apply_block(bc, stackable_block.block)
                {
                    let new_current_block = bc_db_query.get_block_doc_copy();
                    let blockstamp = new_current_block.blockstamp();
                    // Apply db requests
                    bc_db_query
                        .apply(
                            &bc.blocks_databases.blockchain_db,
                            &bc.forks_dbs,
                            bc.currency_params.fork_window_size,
                            None,
                        )
                        .expect("Fatal error : Fail to apply DBWriteRequest !");
                    for query in &wot_dbs_queries {
                        query
                            .apply(&blockstamp, &bc.currency_params, &bc.wot_databases)
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
                } else {
                    warn!("fail to stackable_block({})", stackable_block_number);
                }
            }
            // Save databases
            bc.blocks_databases.save_dbs();
            bc.forks_dbs.save_dbs();
            bc.wot_databases.save_dbs();
            bc.currency_databases.save_dbs(true, true);
            break 'blockchain;
        }
    }
}

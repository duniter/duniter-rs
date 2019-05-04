//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Sub-module managing the reception of messages from the inter-node network layer
//! (received by the intermediaries of events transmitted by the network module).

use crate::*;
use std::ops::Deref;

pub fn receive_bc_documents(bc: &mut BlockchainModule, network_documents: &[BlockchainDocument]) {
    for network_document in network_documents {
        if let BlockchainDocument::Block(ref block_doc) = network_document {
            let block_doc = block_doc.deref();
            receive_blocks(bc, vec![block_doc.deref().clone()]);
        }
    }
}

pub fn receive_blocks(bc: &mut BlockchainModule, blocks: Vec<BlockDocument>) {
    debug!("BlockchainModule : receive_blocks()");
    let mut save_blocks_dbs = false;
    let mut save_wots_dbs = false;
    let mut save_currency_dbs = false;
    for block in blocks.into_iter() {
        let blockstamp = block.blockstamp();
        match check_and_apply_block(bc, block) {
            Ok(check_block_return) => match check_block_return {
                CheckAndApplyBlockReturn::ValidBlock(ValidBlockApplyReqs(
                    bc_db_query,
                    wot_dbs_queries,
                    tx_dbs_queries,
                )) => {
                    let new_current_block = bc_db_query.get_block_doc_copy();
                    bc.current_blockstamp = new_current_block.blockstamp();
                    // Apply db requests
                    bc_db_query
                        .apply(&bc.blocks_databases.blockchain_db, &bc.forks_dbs, None)
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
                    save_blocks_dbs = true;
                    if !wot_dbs_queries.is_empty() {
                        save_wots_dbs = true;
                    }
                    if !tx_dbs_queries.is_empty() {
                        save_currency_dbs = true;
                    }
                    events::sent::send_event(
                        bc,
                        &BlockchainEvent::StackUpValidBlock(Box::new(new_current_block)),
                    );
                }
                CheckAndApplyBlockReturn::ForkBlock => {
                    info!("new fork block({})", blockstamp);
                    if let Ok(Some(new_bc_branch)) = fork_algo::fork_resolution_algo(
                        &bc.forks_dbs,
                        bc.current_blockstamp,
                        &bc.invalid_forks,
                    ) {
                        rollback::apply_rollback(bc, new_bc_branch);
                    }
                }
                CheckAndApplyBlockReturn::OrphanBlock => {
                    debug!("new orphan block({})", blockstamp);
                }
            },
            Err(e) => match e {
                BlockError::VerifyBlockHashsError(_) | BlockError::InvalidBlock(_) => {
                    warn!("InvalidBlock({})", blockstamp.id.0);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::ApplyValidBlockError(e2) => {
                    error!("ApplyValidBlockError({}): {:?}", blockstamp.id.0, e2);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::DALError(e2) => {
                    error!("BlockError::DALError({}): {:?}", blockstamp.id.0, e2);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::AlreadyHaveBlockOrOutForkWindow => {
                    debug!("AlreadyHaveBlockOrOutForkWindow({})", blockstamp.id.0);
                }
            },
        }
    }
    // Save databases
    if save_blocks_dbs {
        bc.blocks_databases.save_dbs();
        bc.forks_dbs.save_dbs();
    }
    if save_wots_dbs {
        bc.wot_databases.save_dbs();
    }
    if save_currency_dbs {
        bc.currency_databases.save_dbs(true, true);
    }
}

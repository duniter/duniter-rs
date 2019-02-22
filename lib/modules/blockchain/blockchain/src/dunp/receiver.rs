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
use dubp_documents::Blockstamp;
use dup_crypto::keys::*;
use durs_wot::{NodeId, WebOfTrust};
use std::collections::HashMap;

pub fn receive_bc_documents<W: WebOfTrust>(
    bc: &mut BlockchainModule,
    network_documents: &[BlockchainDocument],
    mut current_blockstamp: Blockstamp,
    wot_index: &mut HashMap<PubKey, NodeId>,
    wot_db: &BinDB<W>,
) -> Blockstamp {
    for network_document in network_documents {
        if let BlockchainDocument::Block(ref block_doc) = network_document {
            let block_doc = block_doc.deref();
            current_blockstamp = receive_blocks(
                bc,
                vec![Block::NetworkBlock(block_doc.deref().clone())],
                current_blockstamp,
                wot_index,
                wot_db,
            );
        }
    }

    current_blockstamp
}

pub fn receive_blocks<W: WebOfTrust>(
    bc: &mut BlockchainModule,
    blocks: Vec<Block>,
    mut current_blockstamp: Blockstamp,
    wot_index: &mut HashMap<PubKey, NodeId>,
    wot: &BinDB<W>,
) -> Blockstamp {
    debug!("BlockchainModule : receive_blocks()");
    let mut save_blocks_dbs = false;
    let mut save_wots_dbs = false;
    let mut save_currency_dbs = false;
    for block in blocks.into_iter() {
        let _from_network = block.is_from_network();
        let blockstamp = block.blockstamp();
        match check_and_apply_block::<W>(
            &bc.blocks_databases,
            &bc.forks_dbs,
            &bc.wot_databases.certs_db,
            block,
            &current_blockstamp,
            wot_index,
            wot,
        ) {
            Ok(check_block_return) => match check_block_return {
                CheckAndApplyBlockReturn::ValidBlock(ValidBlockApplyReqs(
                    bc_db_query,
                    wot_dbs_queries,
                    tx_dbs_queries,
                )) => {
                    current_blockstamp = blockstamp;
                    // Apply db requests
                    bc_db_query
                        .apply(&bc.blocks_databases.blockchain_db, &bc.forks_dbs, None)
                        .expect("Fatal error : Fail to apply DBWriteRequest !");
                    for query in &wot_dbs_queries {
                        query
                            .apply(&bc.wot_databases, &bc.currency_params)
                            .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
                    }
                    for query in &tx_dbs_queries {
                        query
                            .apply(&bc.currency_databases)
                            .expect("Fatal error : Fail to apply CurrencyDBsWriteRequest !");
                    }
                    save_blocks_dbs = true;
                    if !wot_dbs_queries.is_empty() {
                        save_wots_dbs = true;
                    }
                    if !tx_dbs_queries.is_empty() {
                        save_currency_dbs = true;
                    }
                }
                CheckAndApplyBlockReturn::ForkBlock => {
                    info!("new fork block({})", blockstamp);
                    if let Ok(Some(_new_bc_branch)) = fork_algo::fork_resolution_algo(
                        &bc.forks_dbs,
                        current_blockstamp,
                        &bc.invalid_forks,
                    ) {
                        // TODO apply roolback here
                        rollback::apply_rollback(bc);
                    }
                }
                CheckAndApplyBlockReturn::OrphanBlock => {
                    debug!("new orphan block({})", blockstamp);
                }
            },
            Err(_) => {
                warn!("RefusedBlock({})", blockstamp.id.0);
                crate::events::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
            }
        }
    }
    // Save databases
    if save_blocks_dbs {
        bc.blocks_databases.save_dbs();
    }
    if save_wots_dbs {
        bc.wot_databases.save_dbs();
    }
    if save_currency_dbs {
        bc.currency_databases.save_dbs(true, true);
    }
    current_blockstamp
}

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

//! Sub-module managing the reception of messages from the inter-node network layer
//! (received by the intermediaries of events transmitted by the network module).

use crate::*;
use dubp_documents::documents::UserDocumentDUBP;
use unwrap::unwrap;

pub fn receive_user_documents(_bc: &mut BlockchainModule, network_documents: &[UserDocumentDUBP]) {
    for network_document in network_documents {
        match network_document {
            UserDocumentDUBP::Certification(_) => {}
            UserDocumentDUBP::Identity(_) => {}
            UserDocumentDUBP::Membership(_) => {}
            UserDocumentDUBP::Revocation(_) => {}
            UserDocumentDUBP::Transaction(_) => {}
        }
    }
}

pub fn receive_blocks(bc: &mut BlockchainModule, blocks: Vec<BlockDocument>) {
    debug!("BlockchainModule : receive_blocks({})", blocks.len());
    let mut save_blocks_dbs = false;
    let mut save_wots_dbs = false;
    let mut save_currency_dbs = false;
    let mut first_orphan = true;
    for block in blocks.into_iter() {
        let blockstamp = block.blockstamp();
        match check_and_apply_block(bc, block) {
            Ok(check_block_return) => match check_block_return {
                CheckAndApplyBlockReturn::ValidMainBlock(ValidBlockApplyReqs(
                    bc_db_query,
                    wot_dbs_queries,
                    tx_dbs_queries,
                )) => {
                    let new_current_block = bc_db_query.get_block_doc_copy();
                    bc.current_blockstamp = new_current_block.blockstamp();
                    // Apply db requests
                    bc_db_query
                        .apply(
                            &bc.blocks_databases.blockchain_db,
                            &bc.forks_dbs,
                            unwrap!(bc.currency_params).fork_window_size,
                            None,
                        )
                        .expect("Fatal error : Fail to apply DBWriteRequest !");
                    for query in &wot_dbs_queries {
                        query
                            .apply(&blockstamp, &unwrap!(bc.currency_params), &bc.wot_databases)
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
                    info!("blockchain: new fork block(#{})", blockstamp);
                    bc.forks_dbs.save_dbs();
                    if let Ok(Some(new_bc_branch)) = fork_algo::fork_resolution_algo(
                        &bc.forks_dbs,
                        unwrap!(bc.currency_params).fork_window_size,
                        bc.current_blockstamp,
                        &bc.invalid_forks,
                    ) {
                        info!("blockchain: apply_rollback({:?})", new_bc_branch);
                        rollback::apply_rollback(bc, new_bc_branch);
                    }
                }
                CheckAndApplyBlockReturn::OrphanBlock => {
                    if first_orphan {
                        first_orphan = false;
                        debug!("blockchain: new orphan block(#{})", blockstamp);
                        crate::requests::sent::request_orphan_previous(bc, blockstamp);
                    }
                }
            },
            Err(e) => match e {
                BlockError::VerifyBlockHashError(_) | BlockError::InvalidBlock(_) => {
                    warn!("InvalidBlock(#{})", blockstamp.id.0);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::ApplyValidBlockError(e2) => {
                    error!("ApplyValidBlockError(#{}): {:?}", blockstamp, e2);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::DALError(e2) => {
                    error!("BlockError::DALError(#{}): {:?}", blockstamp, e2);
                    crate::events::sent::send_event(bc, &BlockchainEvent::RefusedBlock(blockstamp));
                }
                BlockError::AlreadyHaveBlock => {
                    debug!("AlreadyHaveBlock(#{})", blockstamp.id);
                }
                BlockError::BlockOrOutForkWindow => {
                    debug!("BlockOrOutForkWindow(#{})", blockstamp);
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

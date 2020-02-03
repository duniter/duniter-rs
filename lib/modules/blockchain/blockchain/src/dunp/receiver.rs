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

use crate::dubp::apply::exec_currency_queries;
use crate::*;
use dubp_common_doc::traits::Document;
use dubp_user_docs::documents::UserDocumentDUBP;
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
    let mut save_dbs = false;
    let mut save_wots_dbs = false;
    let mut first_orphan = true;
    for block in blocks.into_iter() {
        let blockstamp = block.blockstamp();

        // For eventually rollback
        let mut new_bc_branch_opt = None;

        // Open write db transaction
        let db = bc.take_db();
        db.write(|mut w| {
            match check_and_apply_block(bc, &db, &mut w, block) {
                Ok(check_block_return) => match check_block_return {
                    CheckAndApplyBlockReturn::ValidMainBlock(WriteBlockQueries(
                        bc_db_query,
                        wot_dbs_queries,
                        tx_dbs_queries,
                    )) => {
                        let new_current_block = bc_db_query.get_block_doc_copy();
                        bc.current_blockstamp = new_current_block.blockstamp();

                        // Apply db requests
                        bc_db_query.apply(
                            &db,
                            &mut w,
                            &mut bc.fork_tree,
                            unwrap!(bc.currency_params).fork_window_size,
                            None,
                        )?;
                        for query in &wot_dbs_queries {
                            query
                                .apply(&db, &mut w, &blockstamp, &unwrap!(bc.currency_params))
                                .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
                        }
                        exec_currency_queries(&db, &mut w, blockstamp.id, tx_dbs_queries)?;
                        if !wot_dbs_queries.is_empty() {
                            save_wots_dbs = true;
                        }
                        durs_bc_db_writer::blocks::fork_tree::save_fork_tree(
                            &db,
                            &mut w,
                            &bc.fork_tree,
                        )?;
                        save_dbs = true;
                        events::sent::send_event(
                            bc,
                            &BlockchainEvent::StackUpValidBlock(Box::new(new_current_block)),
                        );
                    }
                    CheckAndApplyBlockReturn::ForkBlock => {
                        info!("blockchain: new fork block(#{})", blockstamp);
                        if let Ok(Some(new_bc_branch)) = fork_algo::fork_resolution_algo(
                            &BcDbRwWithWriter { db: &db, w: &w },
                            &bc.fork_tree,
                            unwrap!(bc.currency_params).fork_window_size,
                            bc.current_blockstamp,
                            &bc.invalid_forks,
                        ) {
                            new_bc_branch_opt = Some(new_bc_branch);
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
                    BlockError::InvalidBlock(e) => {
                        warn!("InvalidBlock #{}: {:?}", blockstamp.id.0, e);
                        crate::events::sent::send_event(
                            bc,
                            &BlockchainEvent::RefusedBlock(blockstamp),
                        );
                    }
                    BlockError::ApplyValidBlockError(e2) => {
                        error!("ApplyValidBlockError(#{}): {:?}", blockstamp, e2);
                        crate::events::sent::send_event(
                            bc,
                            &BlockchainEvent::RefusedBlock(blockstamp),
                        );
                    }
                    BlockError::DbError(e2) => {
                        error!("BlockError::DbError(#{}): {:?}", blockstamp, e2);
                        crate::events::sent::send_event(
                            bc,
                            &BlockchainEvent::RefusedBlock(blockstamp),
                        );
                    }
                    BlockError::AlreadyHaveBlock => {
                        debug!("AlreadyHaveBlock(#{})", blockstamp.id);
                    }
                    BlockError::BlockOrOutForkWindow => {
                        debug!("BlockOrOutForkWindow(#{})", blockstamp);
                    }
                },
            }
            Ok(WriteResp::from(w))
        })
        .unwrap_or_else(|_| fatal_error!("Fail to check or apply block: {}.", blockstamp));
        bc.db = Some(db);

        if let Some(new_bc_branch) = new_bc_branch_opt {
            info!("blockchain: apply_rollback({:?})", new_bc_branch);
            rollback::apply_rollback(bc, new_bc_branch);
        }
    }
    // Save databases
    if save_dbs {
        bc.db()
            .save()
            .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
    }
    if save_wots_dbs {
        bc.wot_databases.save_dbs();
    }
}

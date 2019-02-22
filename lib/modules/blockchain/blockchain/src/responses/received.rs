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

//! Sub-module managing the inter-modules responses received.

use crate::*;
use std::ops::Deref;

pub fn receive_response(
    bc: &mut BlockchainModule,
    req_id: ModuleReqId,
    res_content: &DursResContent,
) {
    if let DursResContent::NetworkResponse(ref network_response) = *res_content {
        debug!("BlockchainModule : receive NetworkResponse() !");
        if let Some(request) = bc.pending_network_requests.remove(&req_id) {
            match request {
                OldNetworkRequest::GetConsensus(_) => {
                    if let NetworkResponse::Consensus(_, response) = *network_response.deref() {
                        if let Ok(blockstamp) = response {
                            bc.consensus = blockstamp;
                            if bc.current_blockstamp.id.0 > bc.consensus.id.0 + 2 {
                                // Get last dal_block
                                let last_dal_block_id = BlockId(bc.current_blockstamp.id.0 - 1);
                                let last_dal_block = bc
                                    .blocks_databases
                                    .blockchain_db
                                    .read(|db| db.get(&last_dal_block_id).cloned())
                                    .expect("Fail to read blockchain DB.")
                                    .expect("Fatal error : not foutn last dal block !");
                                revert_block::revert_block(
                                    &last_dal_block,
                                    &mut bc.wot_index,
                                    &bc.wot_databases.wot_db,
                                    &bc.currency_databases
                                        .tx_db
                                        .read(|db| db.clone())
                                        .expect("Fail to read TxDB."),
                                )
                                .expect("Fail to revert block");
                            }
                        }
                    }
                }
                OldNetworkRequest::GetBlocks(_, _, _, _) => {
                    if let NetworkResponse::Chunk(_, _, ref blocks) = *network_response.deref() {
                        let blocks: Vec<Block> = blocks
                            .iter()
                            .map(|b| Block::NetworkBlock(b.deref().clone()))
                            .collect();
                        dunp::receiver::receive_blocks(bc, blocks);
                    }
                }
                _ => {}
            }
        }
    }
}

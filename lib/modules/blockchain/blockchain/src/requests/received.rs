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

//! Sub-module managing the inter-modules requests received.

use crate::*;
use duniter_module::*;
use durs_message::requests::*;

pub fn receive_req(
    bc: &BlockchainModule,
    req_from: ModuleStaticName,
    req_id: ModuleReqId,
    req_content: &DursReqContent,
) {
    if let DursReqContent::BlockchainRequest(ref blockchain_req) = req_content {
        match *blockchain_req {
            BlockchainRequest::CurrentBlock() => {
                debug!("BlockchainModule : receive DALReqBc::CurrentBlock()");

                if let Some(current_block) = readers::block::get_block(
                    &bc.blocks_databases.blockchain_db,
                    None,
                    &bc.current_blockstamp,
                )
                .expect("Fatal error : get_block : fail to read LocalBlockchainV10DB !")
                {
                    debug!(
                        "BlockchainModule : send_req_response(CurrentBlock({}))",
                        bc.current_blockstamp
                    );
                    responses::sent::send_req_response(
                        bc,
                        req_from,
                        req_id,
                        &BlockchainResponse::CurrentBlock(
                            req_id,
                            Box::new(current_block.block),
                            bc.current_blockstamp,
                        ),
                    );
                } else {
                    warn!("BlockchainModule : Req : fail to get current_block in bdd !");
                }
            }
            BlockchainRequest::UIDs(ref pubkeys) => {
                responses::sent::send_req_response(
                    bc,
                    req_from,
                    req_id,
                    &BlockchainResponse::UIDs(
                        req_id,
                        pubkeys
                            .iter()
                            .map(|p| {
                                (
                                    *p,
                                    durs_blockchain_dal::readers::identity::get_uid(
                                        &bc.wot_databases.identities_db,
                                        *p,
                                    )
                                    .expect("Fatal error : get_uid : Fail to read WotV10DB !"),
                                )
                            })
                            .collect(),
                    ),
                );
            }
            _ => {}
        }
    }
}
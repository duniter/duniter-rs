//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! Sub-module managing the WS2Pv1 requests received.

use crate::requests::sent::send_dal_request;
use crate::ws_connections::requests::{WS2Pv1ReqBody, WS2Pv1ReqFullId, WS2Pv1ReqId};
use crate::ws_connections::responses::{WS2Pv1ReqRes, WS2Pv1ReqResBody};
use crate::WS2Pv1Module;
use durs_message::requests::BlockchainRequest;
use durs_network_documents::NodeFullId;

pub fn receive_ws2p_v1_request(
    ws2p_module: &mut WS2Pv1Module,
    from: NodeFullId,
    ws2p_req_id: WS2Pv1ReqId,
    req_boby: WS2Pv1ReqBody,
) {
    let module_req_id_opt = match req_boby {
        WS2Pv1ReqBody::GetCurrent => Some(send_dal_request(
            ws2p_module,
            &BlockchainRequest::CurrentBlock,
        )),
        WS2Pv1ReqBody::GetBlock { number } => Some(send_dal_request(
            ws2p_module,
            &BlockchainRequest::BlockByNumber {
                block_number: number,
            },
        )),
        WS2Pv1ReqBody::GetBlocks { from_number, count } => Some(send_dal_request(
            ws2p_module,
            &BlockchainRequest::Chunk {
                first_block_number: from_number,
                count,
            },
        )),
        WS2Pv1ReqBody::GetRequirementsPending { .. } => {
            crate::ws_connections::responses::sent::send_response(
                ws2p_module,
                from,
                WS2Pv1ReqRes {
                    req_id: ws2p_req_id,
                    body: WS2Pv1ReqResBody::GetRequirementsPending { identities: vec![] },
                },
            );
            None
        }
    };

    if let Some(module_req_id) = module_req_id_opt {
        ws2p_module.pending_received_requests.insert(
            module_req_id,
            WS2Pv1ReqFullId {
                from,
                req_id: ws2p_req_id,
            },
        );
    }
}

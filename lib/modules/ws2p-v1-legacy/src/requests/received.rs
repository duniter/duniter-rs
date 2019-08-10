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

//! Sub-module managing the inter-modules requests received.

use crate::ws2p_db::DbEndpoint;
use crate::ws_connections::requests::{WS2Pv1ReqBody, WS2Pv1ReqId, WS2Pv1Request};
use crate::ws_connections::states::WS2PConnectionState;
use crate::WS2Pv1Module;
use dubp_documents::BlockNumber;
use durs_message::requests::DursReqContent;
use durs_network::requests::OldNetworkRequest;

pub fn receive_req(ws2p_module: &mut WS2Pv1Module, req_content: &DursReqContent) {
    if let DursReqContent::OldNetworkRequest(ref old_net_request) = *req_content {
        match *old_net_request {
            OldNetworkRequest::GetBlocks(ref module_req_full_id, ref count, ref from) => {
                let mut receiver_index = 0;
                let mut real_receiver = None;
                for (ws2p_full_id, DbEndpoint { state, .. }) in ws2p_module.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        if receiver_index == ws2p_module.next_receiver {
                            real_receiver = Some(ws2p_full_id);
                            break;
                        }
                        receiver_index += 1;
                    }
                }
                if real_receiver.is_none() {
                    ws2p_module.next_receiver = 0;
                    for (ws2p_full_id, DbEndpoint { state, .. }) in &ws2p_module.ws2p_endpoints {
                        if let WS2PConnectionState::Established = *state {
                            real_receiver = Some(*ws2p_full_id);
                            break;
                        }
                    }
                } else {
                    ws2p_module.next_receiver += 1;
                }
                if let Some(real_receiver) = real_receiver {
                    debug!("WS2P: send req to: ({:?})", real_receiver);
                    let _blocks_request_result =
                        crate::ws_connections::requests::sent::send_request_to_specific_node(
                            ws2p_module,
                            *module_req_full_id,
                            &real_receiver,
                            &WS2Pv1Request {
                                id: WS2Pv1ReqId::random(),
                                body: WS2Pv1ReqBody::GetBlocks {
                                    count: *count,
                                    from_number: BlockNumber(*from),
                                },
                            },
                        );
                } else {
                    warn!("WS2P: not found peer to send request !");
                }
            }
            OldNetworkRequest::GetEndpoints(ref _request) => {}
            _ => {}
        }
    }
}

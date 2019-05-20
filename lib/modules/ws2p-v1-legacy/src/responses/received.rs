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

//! Sub-module managing the inter-modules responses received.

use crate::ws_connections::responses::{WS2Pv1ReqRes, WS2Pv1ReqResBody};
use crate::*;

pub fn receive_response(
    ws2p_module: &mut WS2Pv1Module,
    req_id: ModuleReqId,
    res_content: &DursResContent,
) {
    if let DursResContent::BlockchainResponse(ref bc_res) = *res_content {
        match *bc_res.deref() {
            BlockchainResponse::CurrentBlockstamp(ref current_blockstamp_) => {
                debug!(
                    "WS2Pv1Module : receive DALResBc::CurrentBlockstamp({})",
                    ws2p_module.current_blockstamp
                );
                ws2p_module.current_blockstamp = *current_blockstamp_;
                if ws2p_module.my_head.is_none() {
                    ws2p_module.my_head = Some(heads::generate_my_head(
                        &ws2p_module.key_pair,
                        ws2p_module.node_id,
                        ws2p_module.soft_name,
                        ws2p_module.soft_version,
                        &ws2p_module.current_blockstamp,
                        None,
                    ));
                }
                let event = NetworkEvent::ReceiveHeads(vec![unwrap!(ws2p_module.my_head.clone())]);
                events::sent::send_network_event(ws2p_module, event);
            }
            BlockchainResponse::UIDs(ref uids) => {
                // Add uids to heads
                for head in ws2p_module.heads_cache.values_mut() {
                    if let Some(uid_option) = uids.get(&head.pubkey()) {
                        if let Some(ref uid) = *uid_option {
                            head.set_uid(uid);
                            ws2p_module
                                .uids_cache
                                .insert(head.pubkey(), uid.to_string());
                        } else {
                            ws2p_module.uids_cache.remove(&head.pubkey());
                        }
                    }
                }
                // Resent heads to other modules
                let event =
                    NetworkEvent::ReceiveHeads(ws2p_module.heads_cache.values().cloned().collect());
                events::sent::send_network_event(ws2p_module, event);
                // Resent to other modules connections that match receive uids
                let events = ws2p_module
                    .ws2p_endpoints
                    .iter()
                    .filter_map(|(node_full_id, DbEndpoint { ep, state, .. })| {
                        if let Some(uid_option) = uids.get(&node_full_id.1) {
                            Some(NetworkEvent::ConnectionStateChange(
                                *node_full_id,
                                *state as u32,
                                uid_option.clone(),
                                ep.get_url(false, false).expect("Endpoint unreachable !"),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();
                events::sent::send_network_events(ws2p_module, events);
            }
            BlockchainResponse::CurrentBlock(ref block_box, _blockstamp) => {
                if let Some(ws2p_req_full_id) =
                    ws2p_module.pending_received_requests.remove(&req_id)
                {
                    ws_connections::responses::sent::send_response(
                        ws2p_module,
                        ws2p_req_full_id.from,
                        WS2Pv1ReqRes {
                            req_id: ws2p_req_full_id.req_id,
                            body: WS2Pv1ReqResBody::GetCurrent(block_box.deref().clone()),
                        },
                    )
                }
            }
            _ => {} // Others BlockchainResponse variants
        }
    }
}

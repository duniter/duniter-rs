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

//! Sub-module managing the WS2Pv1 responses received.

use crate::*;
use dubp_documents::parsers::blocks::parse_json_block_from_serde_value;
use durs_module::ModuleReqFullId;
use durs_network::requests::*;
use durs_network_documents::NodeFullId;

pub fn receive_response(
    ws2p_module: &mut WS2Pv1Module,
    module_req_full_id: ModuleReqFullId,
    ws2p_req_body: WS2Pv1ReqBody,
    recipient_full_id: NodeFullId,
    response: serde_json::Value,
) {
    match ws2p_req_body {
        WS2Pv1ReqBody::GetCurrent => {
            info!(
                "WS2PSignal::ReceiveCurrent({}, {:?})",
                (module_req_full_id.1).0,
                ws2p_req_body
            );
            match parse_json_block_from_serde_value(&response) {
                Ok(block) => {
                    crate::responses::sent::send_network_req_response(
                        ws2p_module,
                        module_req_full_id.0,
                        module_req_full_id.1,
                        NetworkResponse::CurrentBlock(
                            ModuleReqFullId(WS2Pv1Module::name(), module_req_full_id.1),
                            recipient_full_id,
                            Box::new(block),
                        ),
                    );
                }
                Err(e) => warn!("WS2Pv1: receive invalid block: {}.", e),
            }
        }
        WS2Pv1ReqBody::GetBlocks {
            from_number: from, ..
        } => {
            if response.is_array() {
                let mut chunk = Vec::new();
                for json_block in unwrap!(response.as_array()) {
                    match parse_json_block_from_serde_value(json_block) {
                        Ok(block) => chunk.push(block),
                        Err(e) => warn!("WS2Pv1Module: Error : fail to parse json block: {}", e),
                    }
                }
                info!(
                    "WS2PSignal::ReceiveChunk({}, {} blocks from {})",
                    (module_req_full_id.1).0,
                    chunk.len(),
                    from
                );
                debug!("Send chunk to followers : {}", from);
                events::sent::send_network_event(ws2p_module, NetworkEvent::ReceiveBlocks(chunk));
            }
        }
        WS2Pv1ReqBody::GetRequirementsPending { min_cert } => {
            info!(
                "WS2PSignal::ReceiveRequirementsPending({}, {})",
                module_req_full_id.0, min_cert
            );
            debug!("----------------------------------------");
            debug!("-      BEGIN IDENTITIES PENDING        -");
            debug!("----------------------------------------");
            debug!("{:#?}", response);
            debug!("----------------------------------------");
            debug!("-       END IDENTITIES PENDING         -");
            debug!("----------------------------------------");
        }
        _ => {}
    }
}

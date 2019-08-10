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

//! Sub-module managing the WS2Pv1 requests sent.

use super::{WS2Pv1ReqBody, WS2Pv1Request};
use crate::{WS2Pv1Module, WS2Pv1PendingReqInfos};
use durs_module::ModuleReqFullId;
use durs_network_documents::NodeFullId;
use std::time::SystemTime;
use ws::Message;

pub fn send_request_to_specific_node(
    ws2p_module: &mut WS2Pv1Module,
    module_req_full_id: ModuleReqFullId,
    ws2p_full_id: &NodeFullId,
    ws2p_request: &WS2Pv1Request,
) -> ws::Result<()> {
    if let Some(ws) = ws2p_module.websockets.get_mut(ws2p_full_id) {
        let json_req = network_request_to_json(ws2p_request).to_string();
        debug!("send request {} to {}", json_req, ws2p_full_id);
        ws.0.send(Message::text(json_req))?;
        ws2p_module.requests_awaiting_response.insert(
            ws2p_request.id,
            WS2Pv1PendingReqInfos {
                req_body: ws2p_request.body,
                requester_module: module_req_full_id,
                recipient_node: *ws2p_full_id,
                timestamp: SystemTime::now(),
            },
        );
    } else {
        warn!("WS2P: Fail to get mut websocket !");
    }
    Ok(())
}

pub fn network_request_to_json(request: &WS2Pv1Request) -> serde_json::Value {
    let (request_type, request_params) = match request.body {
        WS2Pv1ReqBody::GetCurrent => ("CURRENT", json!({})),
        WS2Pv1ReqBody::GetBlock { ref number } => ("BLOCK_BY_NUMBER", json!({ "number": number })),
        WS2Pv1ReqBody::GetBlocks { count, from_number } => (
            "BLOCKS_CHUNK",
            json!({
                "count": count,
                "fromNumber": from_number
            }),
        ),
        WS2Pv1ReqBody::GetRequirementsPending { min_cert } => (
            "WOT_REQUIREMENTS_OF_PENDING",
            json!({ "minCert": min_cert }),
        ),
    };

    json!({
        "reqId": request.id.to_hyphenated_string(),
        "body" : {
            "name": request_type,
            "params": request_params
        }
    })
}

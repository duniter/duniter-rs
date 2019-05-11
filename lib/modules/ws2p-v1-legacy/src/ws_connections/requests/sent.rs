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

//! Sub-module managing the WS2Pv1 requests sent.

use crate::WS2Pv1Module;
use durs_common_tools::fatal_error;
use durs_network::requests::OldNetworkRequest;
use durs_network_documents::NodeFullId;
use std::time::SystemTime;
use ws::Message;

pub fn send_request_to_specific_node(
    ws2p_module: &mut WS2Pv1Module,
    ws2p_full_id: &NodeFullId,
    ws2p_request: &OldNetworkRequest,
) -> ws::Result<()> {
    if let Some(ws) = ws2p_module.websockets.get_mut(ws2p_full_id) {
        let json_req = network_request_to_json(ws2p_request).to_string();
        debug!("send request {} to {}", json_req, ws2p_full_id);
        ws.0.send(Message::text(json_req))?;
        ws2p_module.requests_awaiting_response.insert(
            ws2p_request.get_req_id(),
            (*ws2p_request, *ws2p_full_id, SystemTime::now()),
        );
    } else {
        warn!("WS2P: Fail to get mut websocket !");
    }
    Ok(())
}

pub fn network_request_to_json(request: &OldNetworkRequest) -> serde_json::Value {
    let (request_id, request_type, request_params) = match *request {
        OldNetworkRequest::GetCurrent(ref req_full_id) => (req_full_id.1, "CURRENT", json!({})),
        OldNetworkRequest::GetBlocks(ref req_full_id, count, from_mumber) => (
            req_full_id.1,
            "BLOCKS_CHUNK",
            json!({
                "count": count,
                "fromNumber": from_mumber
            }),
        ),
        OldNetworkRequest::GetRequirementsPending(ref req_full_id, min_cert) => (
            req_full_id.1,
            "WOT_REQUIREMENTS_OF_PENDING",
            json!({ "minCert": min_cert }),
        ),
        OldNetworkRequest::GetConsensus(_) => {
            fatal_error!("GetConsensus() request must be not convert to json !");
        }
        OldNetworkRequest::GetHeadsCache(_) => {
            fatal_error!("GetHeadsCache() request must be not convert to json !");
        }
        OldNetworkRequest::GetEndpoints(_) => {
            fatal_error!("GetEndpoints() request must be not convert to json !");
        }
    };

    json!({
        "reqId": request_id,
        "body" : {
            "name": request_type,
            "params": request_params
        }
    })
}

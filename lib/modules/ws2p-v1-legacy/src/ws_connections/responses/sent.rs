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

//! Sub-module managing the WS2Pv1 responses sent.

use crate::ws_connections::responses::WS2Pv1ReqRes;
use crate::WS2Pv1Module;
use durs_network_documents::NodeFullId;
use ws::{CloseCode, Message};

pub fn send_response(
    ws2p_module: &mut WS2Pv1Module,
    ws2p_req_from: NodeFullId,
    response: WS2Pv1ReqRes,
) {
    if let Some(ws_sender) = ws2p_module.websockets.get(&ws2p_req_from) {
        let json_response: serde_json::Value = response.into();
        if ws_sender
            .0
            .send(Message::text(json_response.to_string()))
            .is_err()
        {
            let _ = ws_sender
                .0
                .close_with_reason(CloseCode::Error, "Fail to send request response !");
        }
    }
}

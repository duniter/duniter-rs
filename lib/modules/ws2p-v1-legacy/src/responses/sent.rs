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

//! Sub-module managing the inter-modules responses sent.

use crate::*;
use durs_message::*;

pub fn send_network_req_response(
    ws2p_module: &WS2PModule,
    requester: ModuleStaticName,
    req_id: ModuleReqId,
    response: NetworkResponse,
) {
    ws2p_module
        .router_sender
        .send(RouterThreadMessage::ModuleMessage(DursMsg::Response {
            res_from: WS2PModule::name(),
            res_to: requester,
            req_id,
            res_content: DursResContent::NetworkResponse(response),
        }))
        .expect("Fail to send message to router !");
}
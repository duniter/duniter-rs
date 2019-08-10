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

//! Sub-module managing the inter-modules requests sent.

use crate::WS2Pv1Module;
use durs_message::requests::{BlockchainRequest, DursReqContent};
use durs_message::*;
use durs_module::{DursModule, ModuleReqId, ModuleRole, RouterThreadMessage};

pub fn send_dal_request(ws2p_module: &mut WS2Pv1Module, req: &BlockchainRequest) -> ModuleReqId {
    ws2p_module.count_dal_requests += 1;
    if ws2p_module.count_dal_requests == std::u32::MAX {
        ws2p_module.count_dal_requests = 0;
    }

    let req_id = ModuleReqId(ws2p_module.count_dal_requests);

    ws2p_module
        .router_sender
        .send(RouterThreadMessage::ModuleMessage(DursMsg::Request {
            req_from: WS2Pv1Module::name(),
            req_to: ModuleRole::BlockchainDatas,
            req_id,
            req_content: DursReqContent::BlockchainRequest(req.clone()),
        }))
        .expect("Fail to send message to router !");

    req_id
}

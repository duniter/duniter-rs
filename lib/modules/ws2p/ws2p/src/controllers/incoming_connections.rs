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

//! WS2P incoming connections controllers.

use crate::controllers::handler::Ws2pConnectionHandler;
use crate::controllers::*;
use dup_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_message::DursMsg;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use durs_ws2p_protocol::controller::meta_datas::WS2PControllerMetaDatas;
use durs_ws2p_protocol::controller::{WS2PController, WS2PControllerId};
use durs_ws2p_protocol::orchestrator::OrchestratorMsg;
use durs_ws2p_protocol::MySelfWs2pNode;
use std::fmt::Debug;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use ws::deflate::DeflateBuilder;
use ws::listen;

/// Listen on WSPv2 host:port
pub fn listen_on_ws2p_v2_endpoint<A: ToSocketAddrs + Debug>(
    currency: &CurrencyName,
    orchestrator_sender: &mpsc::Sender<OrchestratorMsg<DursMsg>>,
    self_node: &MySelfWs2pNode,
    addr: A,
) -> ws::Result<()> {
    // Connect to websocket
    listen(addr, move |ws| {
        match WS2PController::<DursMsg>::try_new(
            WS2PControllerId::Incoming,
            WS2PControllerMetaDatas::new(
                Hash::random(),
                WS2Pv2ConnectType::Incoming,
                currency.clone(),
                self_node.clone(),
            ),
            orchestrator_sender.clone(),
        ) {
            Ok(controller) => DeflateBuilder::new().build(Ws2pConnectionHandler {
                ws: WsSender(ws),
                remote_addr_opt: None,
                controller,
            }),
            Err(_e) => fatal_error!("WS2P Orchestrator unreachable"),
        }
    })
}

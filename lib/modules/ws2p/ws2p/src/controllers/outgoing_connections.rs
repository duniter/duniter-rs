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

//! WS2P outgoing connections controllers.

use crate::controllers::handler::Ws2pConnectionHandler;
use crate::controllers::*;
use dup_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_message::DursMsg;
use durs_network_documents::network_endpoint::EndpointEnum;
use durs_network_documents::NodeFullId;
use durs_ws2p_protocol::controller::meta_datas::WS2PControllerMetaDatas;
use durs_ws2p_protocol::controller::{WS2PController, WS2PControllerId};
use durs_ws2p_protocol::orchestrator::OrchestratorMsg;
use durs_ws2p_protocol::MySelfWs2pNode;
use ws::connect;
use ws::deflate::DeflateBuilder;
//use durs_network::*;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use std::sync::mpsc;

/// Connect to WSPv2 Endpoint
pub fn connect_to_ws2p_v2_endpoint(
    currency: &CurrencyName,
    orchestrator_sender: &mpsc::Sender<OrchestratorMsg<DursMsg>>,
    self_node: &MySelfWs2pNode,
    expected_remote_full_id: Option<NodeFullId>,
    endpoint: &EndpointEnum,
) -> ws::Result<()> {
    // Get endpoint url
    let ws_url = endpoint.get_url(true, false).expect("Endpoint unreachable");

    // Log
    info!("Try connection to {} ...", ws_url);

    // Connect to websocket
    connect(ws_url, move |ws| {
        match WS2PController::<DursMsg>::try_new(
            WS2PControllerId::Outgoing {
                expected_remote_full_id,
            },
            WS2PControllerMetaDatas::new(
                Hash::random(),
                WS2Pv2ConnectType::OutgoingServer,
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
            Err(_e) => fatal_error!("WS2P Service unreachable"),
        }
    })
}

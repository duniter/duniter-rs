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

//! WS2P outgoing connections controllers.

use dubp_documents::CurrencyName;
//use duniter_module::ModuleReqId;
use controllers::handler::Ws2pConnectionHandler;
use controllers::ws::connect;
use controllers::ws::deflate::DeflateBuilder;
use controllers::*;
use durs_network_documents::network_endpoint::EndpointEnum;
use durs_network_documents::NodeFullId;
use services::*;
//use duniter_network::*;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use std::sync::mpsc;

/// Connect to WSPv2 Endpoint
pub fn connect_to_ws2p_v2_endpoint(
    currency: &CurrencyName,
    service_sender: &mpsc::Sender<Ws2pServiceSender>,
    self_node: &MySelfWs2pNode,
    expected_remote_full_id: Option<NodeFullId>,
    endpoint: &EndpointEnum,
) -> ws::Result<()> {
    // Get endpoint url
    let ws_url = endpoint.get_url(true, false).expect("Endpoint unreachable");

    // Create Ws2pConnectionDatas
    let mut conn_meta_datas = Ws2pConnectionDatas::new(WS2Pv2ConnectType::Classic);

    // Indicate expected remote_full_id
    conn_meta_datas.remote_full_id = expected_remote_full_id;

    // Log
    info!("Try connection to {} ...", ws_url);
    println!("DEBUG: Try connection to {} ...", ws_url);

    // Connect to websocket
    connect(ws_url, move |ws| {
        DeflateBuilder::new().build(
            Ws2pConnectionHandler::new(
                WsSender(ws),
                service_sender.clone(),
                currency.clone(),
                self_node.clone(),
                conn_meta_datas.clone(),
            )
            .expect("WS2P Service unrechable"),
        )
    })
}

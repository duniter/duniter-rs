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

//! Controller process event ws connection opened

use super::{
    WS2PConnectionState, WS2PController, WS2PControllerProcessError, WebsocketActionOrder,
};
use crate::websocket::{WebsocketAction, WebsocketMessage};
use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
use durs_ws2p_messages::v2::connect::generate_connect_message;
use durs_ws2p_messages::v2::payload_container::WS2Pv2MessagePayload;
use durs_ws2p_messages::v2::WS2Pv2Message;
use log::error;
use std::net::SocketAddr;

pub fn process<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    remote_addr_opt: Option<SocketAddr>,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    log::debug!("open websocket from {}", print_opt_addr(remote_addr_opt));

    // Update connection state
    controller.update_conn_state(WS2PConnectionState::TryToSendConnectMsg)?;

    // Generate connect message
    let connect_msg = generate_connect_message(
        controller.meta_datas.connect_type,
        controller.meta_datas.local_node.my_features.clone(),
        controller.meta_datas.challenge,
        None,
    );

    // Encapsulate and binarize connect message
    if let Ok((_ws2p_full_msg, bin_connect_msg)) = WS2Pv2Message::encapsulate_payload(
        controller.meta_datas.currency.clone(),
        controller.meta_datas.local_node.my_node_id,
        controller.meta_datas.local_node.my_key_pair,
        WS2Pv2MessagePayload::Connect(Box::new(connect_msg)),
    ) {
        // Order the sending of a CONNECT message
        Ok(Some(WebsocketActionOrder {
            ws_action: WebsocketAction::SendMessage {
                msg: WebsocketMessage::Bin(bin_connect_msg),
            },
            new_state_if_success: Some(WS2PConnectionState::WaitingConnectMsg),
            new_state_if_fail: WS2PConnectionState::Unreachable,
        }))
    } else {
        fatal_error!("Dev error: Fail to sign own connect message !")
    }
}

fn print_opt_addr(addr: Option<SocketAddr>) -> String {
    match addr {
        Some(addr) => format!("{}", addr),
        None => String::from(""),
    }
}

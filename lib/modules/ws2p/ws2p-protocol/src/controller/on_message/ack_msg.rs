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

//! Sub-module process reception of ACK message

use crate::connection_state::WS2PConnectionState;
use crate::controller::{WS2PController, WS2PControllerProcessError, WebsocketActionOrder};
use crate::websocket::{WebsocketAction, WebsocketMessage};
use dup_crypto::hashs::Hash;
use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
use durs_ws2p_messages::v2::ok::WS2Pv2OkMsg;
use durs_ws2p_messages::v2::payload_container::WS2Pv2MessagePayload;
use durs_ws2p_messages::v2::WS2Pv2Message;
use log::error;

/// Process WS2P v2+ ACK Message
pub fn process_ws2p_v2p_ack_msg<M: ModuleMessage>(
    controller: &mut WS2PController<M>, // controller contains original challenge
    ack_msg_challenge: Hash,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    log::debug!("Receive ACK message !");

    match controller.meta_datas.state {
        WS2PConnectionState::OkMsgOkWaitingAckMsg => {
            // already sent ack message and received ok response
            process(
                controller,
                ack_msg_challenge,
                WS2PConnectionState::Established,
            )
        }
        WS2PConnectionState::ConnectMessOk => {
            // ack message not yet sent
            process(controller, ack_msg_challenge, WS2PConnectionState::AckMsgOk)
        }
        _ => Ok(super::close_with_reason(
            "Unexpected ACK message !",
            WS2PConnectionState::Denial,
        )),
    }
}

#[inline]
// process and apply given status in case of success
fn process<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    ack_msg_challenge: Hash,
    success_status: WS2PConnectionState,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    if controller.meta_datas.challenge != ack_msg_challenge {
        controller
            .update_conn_state(WS2PConnectionState::Denial)
            .map(|_| None)
    } else {
        Ok(Some(send_ok_msg(controller, success_status)))
    }
}

// send ok message
fn send_ok_msg<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    success_status: WS2PConnectionState,
) -> WebsocketActionOrder {
    // generate empty Ok message
    let ok_msg = WS2Pv2OkMsg::default();

    // Encapsulate and binarize OK message
    if let Ok((_, bin_ok_msg)) = WS2Pv2Message::encapsulate_payload(
        controller.meta_datas.currency.clone(),
        controller.meta_datas.local_node.my_node_id,
        &controller.meta_datas.signator,
        WS2Pv2MessagePayload::Ok(ok_msg),
    ) {
        // Order the sending of a OK message
        WebsocketActionOrder {
            ws_action: WebsocketAction::SendMessage {
                msg: WebsocketMessage::Bin(bin_ok_msg),
            },
            new_state_if_success: Some(success_status),
            new_state_if_fail: WS2PConnectionState::Unreachable,
        }
    } else {
        fatal_error!("Dev error: Fail to sign own ok message !");
    }
}

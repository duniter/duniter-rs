// Copyright (C) 2018 The Durs Project Developers.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Process WS2P ACK message.

use crate::controllers::handler::*;
use crate::controllers::*;
use dup_crypto::hashs::Hash;
use durs_common_tools::fatal_error;
use durs_ws2p_messages::v2::ok::WS2Pv2OkMsg;
use ws::CloseCode;

/// Process WS2pv2 ACK Message
pub fn process_ws2p_v2_ack_msg(
    handler: &mut Ws2pConnectionHandler, // handler contains original challenge
    ack_msg_challenge: Hash,
) {
    debug!("Receive ACK message !");

    match handler.conn_datas.state {
        WS2PConnectionState::OkMsgOkWaitingAckMsg => {
            // already sent ack message and received ok response
            process(handler, ack_msg_challenge, WS2PConnectionState::Established);
        }
        WS2PConnectionState::ConnectMessOk => {
            // ack message not yet sent
            process(handler, ack_msg_challenge, WS2PConnectionState::AckMsgOk);
        }
        _ => {
            let _ = handler
                .ws
                .0
                .close_with_reason(CloseCode::Invalid, "Unexpected ACK message !");
        }
    }
}

#[inline]
// process and apply given status in case of success
fn process(
    handler: &mut Ws2pConnectionHandler,
    ack_msg_challenge: Hash,
    success_status: WS2PConnectionState,
) {
    if handler.conn_datas.challenge != ack_msg_challenge {
        handler.update_status(WS2PConnectionState::Denial);
    } else {
        handler.update_status(success_status);
        send_ok_msg(handler);
    }
}

// send ok message
fn send_ok_msg(handler: &mut Ws2pConnectionHandler) {
    // generate empty Ok message
    let ok_msg: WS2Pv2OkMsg = Default::default();

    // Encapsulate and binarize OK message
    if let Ok((_, bin_ok_msg)) = WS2Pv2Message::encapsulate_payload(
        handler.currency.clone(),
        handler.local_node.my_node_id,
        handler.local_node.my_key_pair,
        WS2Pv2MessagePayload::Ok(ok_msg),
    ) {
        // Send Ok Message
        match handler.ws.0.send(Message::binary(bin_ok_msg)) {
            Ok(()) => {}
            Err(_) => {
                handler.conn_datas.state = WS2PConnectionState::Unreachable;
                let _ = handler
                    .ws
                    .0
                    .close_with_reason(CloseCode::Error, "Fail to send Ok message !");
            }
        }
    } else {
        fatal_error!("Dev error: Fail to sign own ok message !");
    }
}

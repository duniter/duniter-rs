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

//! Controller process event ws message received

mod ack_msg;
mod connect_msg;
mod ok_msg;
mod secret_flags;

use super::{WS2PController, WS2PControllerProcessError, WebsocketActionOrder};
use crate::connection_state::WS2PConnectionState;
use crate::constants;
use crate::controller::WS2PControllerEvent;
use crate::websocket::{WebsocketAction, WebsocketMessage};
use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::payload_container::WS2Pv2MessagePayload;
use durs_ws2p_messages::WS2PMessage;
use log::error;
use std::ops::Deref;
use std::thread;
use std::time::{Duration, SystemTime};

pub fn process<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    msg: WebsocketMessage,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    // Update last_mess_time
    controller.meta_datas.last_mess_time = SystemTime::now();

    // Spam ?
    if SystemTime::now()
        .duration_since(controller.meta_datas.last_mess_time)
        .unwrap()
        > Duration::new(*constants::WS2P_SPAM_INTERVAL_IN_MILLI_SECS, 0)
    {
        if controller.meta_datas.spam_interval {
            controller.meta_datas.spam_counter += 1;
        } else {
            controller.meta_datas.spam_interval = true;
            controller.meta_datas.spam_counter = 2;
        }
    } else {
        controller.meta_datas.spam_interval = false;
        controller.meta_datas.spam_counter = 0;
    }
    // Spam ?
    if controller.meta_datas.spam_counter >= *constants::WS2P_SPAM_LIMIT {
        thread::sleep(Duration::from_millis(
            *constants::WS2P_SPAM_SLEEP_TIME_IN_SEC,
        ));
        controller.meta_datas.last_mess_time = SystemTime::now();
        return Ok(None);
    }

    if let WebsocketMessage::Bin(bin_msg) = msg {
        log::debug!("Receive new bin message there is not a spam !");
        match WS2PMessage::parse_and_check_bin_message(&bin_msg) {
            Ok(valid_msg) => match valid_msg {
                WS2PMessage::V2(ref msg_v2) => {
                    match msg_v2.payload {
                        WS2Pv2MessagePayload::Connect(ref box_connect_msg) => {
                            let connect_msg = box_connect_msg.deref();
                            // Get remote node id
                            let remote_full_id =
                                NodeFullId(msg_v2.issuer_node_id, msg_v2.issuer_pubkey);
                            // Process connect message
                            connect_msg::process_ws2p_v2p_connect_msg(
                                controller,
                                remote_full_id,
                                connect_msg,
                            )
                        }
                        WS2Pv2MessagePayload::Ack {
                            challenge: ack_msg_challenge,
                        } => {
                            // Process ack message
                            ack_msg::process_ws2p_v2p_ack_msg(controller, ack_msg_challenge)
                        }
                        WS2Pv2MessagePayload::SecretFlags(ref secret_flags) => {
                            secret_flags::process_ws2p_v2p_secret_flags_msg(
                                controller,
                                secret_flags,
                            )
                        }
                        WS2Pv2MessagePayload::Ok(_) => {
                            // Process ok message
                            ok_msg::process_ws2p_v2p_ok_msg(controller)
                        }
                        WS2Pv2MessagePayload::Ko(_) => Ok(close_with_reason(
                            "Receive Ko message !",
                            WS2PConnectionState::Denial,
                        )),
                        _ => {
                            if let WS2PConnectionState::Established = controller.meta_datas.state {
                                controller
                                    .send_event(WS2PControllerEvent::RecvValidMsg {
                                        ws2p_msg: valid_msg,
                                    })
                                    .map(|_| None)
                            } else {
                                Ok(close_with_reason(
                                    "Receive datas message on negociation !",
                                    WS2PConnectionState::Denial,
                                ))
                            }
                        }
                    }
                }
                WS2PMessage::_V0 | WS2PMessage::_V1 => {
                    fatal_error!("Dev error: must not use WS2PMessage version < 2 in WS2Pv2+ !")
                }
            },
            Err(ws2p_msg_err) => {
                log::warn!("Message is invalid : {:?}", ws2p_msg_err);
                controller.meta_datas.count_invalid_msgs += 1;
                if controller.meta_datas.count_invalid_msgs >= *constants::WS2P_INVALID_MSGS_LIMIT {
                    Ok(close_with_reason(
                        "Receive several invalid messages !",
                        WS2PConnectionState::Denial,
                    ))
                } else {
                    Ok(None)
                }
            }
        }
    } else {
        Ok(close_with_reason(
            "Receive str message !",
            WS2PConnectionState::Denial,
        ))
    }
}

fn close_with_reason(reason: &str, new_state: WS2PConnectionState) -> Option<WebsocketActionOrder> {
    Some(WebsocketActionOrder {
        ws_action: WebsocketAction::CloseConnection {
            reason: Some(reason.to_owned()),
        },
        new_state_if_success: Some(new_state),
        new_state_if_fail: new_state,
    })
}

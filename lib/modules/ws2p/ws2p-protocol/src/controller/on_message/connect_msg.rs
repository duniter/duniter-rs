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

//! Sub-module process reception of CONNECT message

use crate::connection_state::WS2PConnectionState;
use crate::controller::meta_datas::Ws2pRemoteNodeDatas;
use crate::controller::{WS2PController, WS2PControllerProcessError, WebsocketActionOrder};
use crate::websocket::{WebsocketAction, WebsocketMessage};
use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::connect::{WS2Pv2ConnectMsg, WS2Pv2ConnectType};
use durs_ws2p_messages::v2::payload_container::WS2Pv2MessagePayload;
use durs_ws2p_messages::v2::WS2Pv2Message;
use log::error;
use unwrap::unwrap;

/// Process WS2P v2+ CONNECT Message
pub fn process_ws2p_v2p_connect_msg<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    remote_full_id: NodeFullId,
    connect_msg: &WS2Pv2ConnectMsg,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    log::debug!("Receive CONNECT message !");

    // Get remote node datas
    let remote_challenge = connect_msg.challenge;
    let remote_node_datas = Ws2pRemoteNodeDatas {
        challenge: connect_msg.challenge,
        current_blockstamp: None,
        peer_card: None,
        remote_full_id,
    };

    if let WS2PConnectionState::WaitingConnectMsg = controller.meta_datas.state {
        // Check remote node datas
        if let WS2Pv2ConnectType::Incoming = controller.meta_datas.connect_type {
            controller.meta_datas.remote_node = Some(remote_node_datas);
            // Get remote_connect_type
            controller.meta_datas.remote_connect_type = Some(WS2Pv2ConnectType::from_flags(
                &connect_msg.flags_queries,
                connect_msg.chunkstamp,
            ));
        } else {
            let expected_full_id = unwrap!(controller.id.expected_remote_full_id());
            if remote_full_id == expected_full_id {
                controller.meta_datas.remote_node = Some(remote_node_datas);
            } else {
                return Ok(super::close_with_reason(
                    "Unexpected PUBKEY or NODE_ID !",
                    WS2PConnectionState::Denial,
                ));
            }
            // Flags not allowed from incoming node
            if !connect_msg.flags_queries.is_empty() {
                super::close_with_reason(
                    "Unexpected CONNECT FLAGS from incoming node. !",
                    WS2PConnectionState::Denial,
                );
            }
            // Get remote_connect_type
            controller.meta_datas.remote_connect_type = Some(WS2Pv2ConnectType::Incoming);
        }
    } else {
        super::close_with_reason("Unexpected CONNECT message !", WS2PConnectionState::Denial);
    }

    // Check features compatibility
    match controller
        .meta_datas
        .local_node
        .my_features
        .check_features_compatibility(connect_msg.api_features)
    {
        Ok(merged_features) => controller.meta_datas.features = Some(merged_features),
        Err(_) => {
            super::close_with_reason("Unsupported features !", WS2PConnectionState::Denial);
        }
    }

    // Encapsulate and binarize ACK message
    if let Ok((_, bin_ack_msg)) = WS2Pv2Message::encapsulate_payload(
        controller.meta_datas.currency.clone(),
        controller.meta_datas.local_node.my_node_id,
        &controller.meta_datas.signator,
        WS2Pv2MessagePayload::Ack {
            challenge: remote_challenge,
        },
    ) {
        // Order the sending of a OK message
        Ok(Some(WebsocketActionOrder {
            ws_action: WebsocketAction::SendMessage {
                msg: WebsocketMessage::Bin(bin_ack_msg),
            },
            new_state_if_success: Some(WS2PConnectionState::ConnectMessOk),
            new_state_if_fail: WS2PConnectionState::Unreachable,
        }))
    } else {
        fatal_error!("Dev error: Fail to sign own ack message !")
    }
}

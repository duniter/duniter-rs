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

//! Process WS2P CONNECT mesage.

use controllers::handler::*;
use controllers::ws::CloseCode;
use controllers::*;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectMsg;
//use services::Ws2pServiceSender;
//use std::sync::mpsc;

/// Process WS2pv2 CONNECT Message
pub fn process_ws2p_v2_connect_msg(
    handler: &mut Ws2pConnectionHandler,
    remote_full_id: NodeFullId,
    connect_msg: &WS2Pv2ConnectMsg,
) {
    println!("DEBUG: Receive CONNECT message !");

    // Get remote node datas
    let remote_challenge = connect_msg.challenge;
    let remote_node_datas = Ws2pRemoteNodeDatas {
        challenge: connect_msg.challenge,
        peer_card: None,
        current_blockstamp: None,
    };
    if let WS2PConnectionState::WaitingConnectMess = handler.conn_datas.state {
        // Check remote node datas
        if let WS2Pv2ConnectType::Incoming = handler.conn_datas.connect_type {
            handler.conn_datas.remote_full_id = Some(remote_full_id);
            handler.conn_datas.remote_datas = Some(remote_node_datas);
            // Get remote_connect_type
            handler.conn_datas.remote_connect_type = Some(WS2Pv2ConnectType::from_flags(
                &connect_msg.flags_queries,
                connect_msg.chunkstamp,
            ));
        } else {
            let expected_full_id = handler
                .conn_datas
                .remote_full_id
                .expect("Outcoming connection must have expected remote node full id !");
            if remote_full_id == expected_full_id {
                handler.conn_datas.remote_datas = Some(remote_node_datas);
            } else {
                let _ = handler
                    .ws
                    .0
                    .close_with_reason(CloseCode::Invalid, "Unexpected PUBKEY or NODE_ID !");
            }
            // Flags not allowed from incoming node
            if !connect_msg.flags_queries.is_empty() {
                let _ = handler.ws.0.close_with_reason(
                    CloseCode::Invalid,
                    "Unexpected CONNECT FLAGS from incoming node. !",
                );
            }
            // Get remote_connect_type
            handler.conn_datas.remote_connect_type = Some(WS2Pv2ConnectType::Incoming);
        }
    } else {
        let _ = handler
            .ws
            .0
            .close_with_reason(CloseCode::Invalid, "Unexpected CONNECT message !");
    }
    // Check features compatibility
    match handler
        .local_node
        .my_features
        .check_features_compatibility(&connect_msg.api_features)
    {
        Ok(merged_features) => handler.conn_datas.features = Some(merged_features),
        Err(_) => {
            let _ = handler
                .ws
                .0
                .close_with_reason(CloseCode::Unsupported, "Unsupported features !");
        }
    }

    // Update Status to ConnectMessOk
    handler.conn_datas.state = WS2PConnectionState::ConnectMessOk;
    handler.send_new_conn_state_to_service();

    // Encapsulate and binarize ACK message
    let (_, bin_ack_msg) = WS2Pv2Message::encapsulate_payload(
        handler.currency.clone(),
        handler.local_node.my_node_id,
        handler.local_node.my_key_pair,
        WS2Pv2MessagePayload::Ack(remote_challenge),
    )
    .expect("WS2P : Fail to sign own ack message !");
    // Send ACk Message
    match handler.ws.0.send(Message::binary(bin_ack_msg)) {
        Ok(()) => {
            // Update state
            handler.conn_datas.state = WS2PConnectionState::ConnectMessOk;
        }
        Err(_) => {
            handler.conn_datas.state = WS2PConnectionState::Unreachable;
            let _ = handler
                .ws
                .0
                .close_with_reason(CloseCode::Error, "Fail to send ACk message !");
        }
    }
}

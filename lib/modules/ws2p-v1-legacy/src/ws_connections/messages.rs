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

//! Define ws2p connections messages.

use super::*;
use crate::ws_connections::requests::WS2Pv1ReqBody;
use dubp_block_doc::DocumentDUBP;
use durs_network_documents::NodeFullId;
use ws::Message;

#[derive(Debug)]
/// WS2Pv1 Message
pub struct WS2Pv1Msg {
    pub from: NodeFullId,
    pub payload: WS2Pv1MsgPayload,
}

#[derive(Debug)]
/// WS2Pv1 Message payload
pub enum WS2Pv1MsgPayload {
    FailOpenWS,
    WrongUrl,
    FailToSplitWS,
    TryToSendConnectMess,
    FailSendConnectMess,
    WebsocketOk(WsSender),
    NegociationTimeout,
    ValidConnectMessage(String, WS2PConnectionState),
    ValidAckMessage(String, WS2PConnectionState),
    ValidOk(WS2PConnectionState),
    Request {
        req_id: WS2Pv1ReqId,
        body: WS2Pv1ReqBody,
    },
    PeerCard(serde_json::Value, Vec<EndpointV1>),
    Heads(Vec<serde_json::Value>),
    Document(DocumentDUBP),
    ReqResponse(WS2Pv1ReqId, serde_json::Value),
    InvalidMessage,
    WrongFormatMessage,
    UnknowMessage,
    Timeout,
    Close,
}

pub fn generate_connect_message(
    currency: &str,
    key_pair: KeyPairEnum,
    challenge: String,
) -> Message {
    // Create CONNECT Message
    let mut connect_message = WS2PConnectMessageV1 {
        currency: String::from(currency),
        pubkey: key_pair.public_key(),
        challenge,
        signature: None,
    };
    connect_message.signature = Some(connect_message.sign(key_pair));
    Message::text(
        serde_json::to_string(&connect_message).expect("Fail to serialize CONNECT message !"),
    )
}

pub fn ws2p_recv_message_pretreatment(
    ws2p_module: &mut WS2Pv1Module,
    message: WS2Pv1Msg,
) -> WS2PSignal {
    check_timeout_requests(ws2p_module);

    let ws2p_full_id = message.from;
    match message.payload {
        WS2Pv1MsgPayload::WrongUrl
        | WS2Pv1MsgPayload::FailOpenWS
        | WS2Pv1MsgPayload::FailToSplitWS => {
            let dal_ep = ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !");
            dal_ep.state = WS2PConnectionState::WSError;
            dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
            return WS2PSignal::WSError(ws2p_full_id);
        }
        WS2Pv1MsgPayload::TryToSendConnectMess => {
            ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !")
                .state = WS2PConnectionState::TryToSendConnectMess;
        }
        WS2Pv1MsgPayload::FailSendConnectMess => {
            let dal_ep = ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !");
            dal_ep.state = WS2PConnectionState::Unreachable;
            dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
        }
        WS2Pv1MsgPayload::WebsocketOk(sender) => {
            ws2p_module.websockets.insert(ws2p_full_id, sender);
        }
        WS2Pv1MsgPayload::ValidConnectMessage(response, new_con_state) => {
            ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !")
                .state = new_con_state;
            debug!("Send: {:#?}", response);
            if let Some(websocket) = ws2p_module.websockets.get_mut(&ws2p_full_id) {
                if websocket.0.send(Message::text(response)).is_err() {
                    return WS2PSignal::WSError(ws2p_full_id);
                }
            } else {
                // Connection closed by remote peer
                let dal_ep = ws2p_module
                    .ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !");
                dal_ep.state = WS2PConnectionState::Close;
                dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
            }
        }
        WS2Pv1MsgPayload::ValidAckMessage(response, new_con_state) => {
            ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !")
                .state = new_con_state;
            if let WS2PConnectionState::AckMessOk = ws2p_module.ws2p_endpoints[&ws2p_full_id].state
            {
                debug!("Send: {:#?}", response);
                if let Some(websocket) = ws2p_module.websockets.get_mut(&ws2p_full_id) {
                    if websocket.0.send(Message::text(response)).is_err() {
                        return WS2PSignal::WSError(ws2p_full_id);
                    }
                } else {
                    fatal_error!("Fatal error : no websocket for {} !", ws2p_full_id);
                }
            }
        }
        WS2Pv1MsgPayload::ValidOk(new_con_state) => {
            ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !")
                .state = new_con_state;
            let mut close_conn = false;
            let signal = match ws2p_module.ws2p_endpoints[&ws2p_full_id].state {
                WS2PConnectionState::OkMessOkWaitingAckMess => WS2PSignal::Empty,
                WS2PConnectionState::Established => WS2PSignal::ConnectionEstablished(ws2p_full_id),
                _ => {
                    close_conn = true;
                    WS2PSignal::Empty
                }
            };
            if close_conn {
                close_connection(
                    ws2p_module,
                    &ws2p_full_id,
                    WS2PCloseConnectionReason::Unknow,
                );
            }

            return signal;
        }
        WS2Pv1MsgPayload::Request { req_id, body } => {
            return WS2PSignal::Request {
                from: ws2p_full_id,
                req_id,
                body,
            };
        }
        WS2Pv1MsgPayload::PeerCard(body, ws2p_endpoints) => {
            return WS2PSignal::PeerCard(ws2p_full_id, body, ws2p_endpoints);
        }
        WS2Pv1MsgPayload::Heads(heads) => {
            let mut applied_heads = Vec::with_capacity(heads.len());
            for head in heads {
                if let Ok(head) = NetworkHead::from_json_value(&head) {
                    if head.verify()
                        && (ws2p_module.my_head.is_none()
                            || head.node_full_id()
                                != ws2p_module
                                    .my_head
                                    .clone()
                                    .expect("WS2P: Fail to clone my_head")
                                    .node_full_id())
                        && head.apply(&mut ws2p_module.heads_cache)
                    {
                        applied_heads.push(head);
                    }
                }
            }
            return WS2PSignal::Heads(ws2p_full_id, applied_heads);
        }
        WS2Pv1MsgPayload::Document(doc) => match doc {
            DocumentDUBP::Block(block_doc) => {
                return WS2PSignal::Blocks(ws2p_full_id, vec![block_doc.deref().clone()])
            }
            DocumentDUBP::UserDocument(user_doc) => {
                return WS2PSignal::UserDocuments(ws2p_full_id, vec![user_doc]);
            }
        },
        WS2Pv1MsgPayload::ReqResponse(ws2p_req_id, response) => {
            if let Some(WS2Pv1PendingReqInfos {
                ref requester_module,
                ref req_body,
                ref recipient_node,
                ..
            }) = ws2p_module.requests_awaiting_response.remove(&ws2p_req_id)
            {
                return WS2PSignal::ReqResponse(
                    *requester_module,
                    *req_body,
                    *recipient_node,
                    response,
                );
            }
        }
        WS2Pv1MsgPayload::NegociationTimeout => {
            match ws2p_module.ws2p_endpoints[&ws2p_full_id].state {
                WS2PConnectionState::AckMessOk | WS2PConnectionState::ConnectMessOk => {
                    ws2p_module
                        .ws2p_endpoints
                        .get_mut(&ws2p_full_id)
                        .expect("WS2P: Fail to get mut ep !")
                        .state = WS2PConnectionState::Denial
                }
                WS2PConnectionState::WaitingConnectMess => {
                    ws2p_module
                        .ws2p_endpoints
                        .get_mut(&ws2p_full_id)
                        .expect("WS2P: Fail to get mut ep !")
                        .state = WS2PConnectionState::NoResponse
                }
                _ => {
                    let dal_ep = ws2p_module
                        .ws2p_endpoints
                        .get_mut(&ws2p_full_id)
                        .expect("WS2P: Fail to get mut ep !");
                    dal_ep.state = WS2PConnectionState::Unreachable;
                    dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
                }
            }
            close_connection(
                ws2p_module,
                &ws2p_full_id,
                WS2PCloseConnectionReason::NegociationTimeout,
            );
            return WS2PSignal::NegociationTimeout(ws2p_full_id);
        }
        WS2Pv1MsgPayload::Timeout => {
            close_connection(
                ws2p_module,
                &ws2p_full_id,
                WS2PCloseConnectionReason::Timeout,
            );
            return WS2PSignal::Timeout(ws2p_full_id);
        }
        WS2Pv1MsgPayload::UnknowMessage => {}
        WS2Pv1MsgPayload::WrongFormatMessage => warn!(
            "WS2P : Receive Wrong Format Message from {}.",
            &ws2p_full_id.1
        ),
        WS2Pv1MsgPayload::InvalidMessage => return WS2PSignal::Empty,
        WS2Pv1MsgPayload::Close => close_connection(
            ws2p_module,
            &ws2p_full_id,
            WS2PCloseConnectionReason::AuthMessInvalidSig,
        ),
    }
    let connections_count = ws2p_module.websockets.len();
    if connections_count == 0 {
        return WS2PSignal::NoConnection;
    }
    WS2PSignal::Empty
}

fn check_timeout_requests(ws2p_module: &mut WS2Pv1Module) {
    // Detect timeout requests
    let mut requests_timeout = Vec::new();

    for (ws2p_req_id, pending_req_infos) in ws2p_module.requests_awaiting_response.iter() {
        if unwrap!(SystemTime::now().duration_since(pending_req_infos.timestamp))
            > Duration::from_secs(*WS2P_V1_REQUESTS_TIMEOUT_IN_SECS)
        {
            requests_timeout.push(*ws2p_req_id);
            warn!(
                "request timeout : {:?} (sent to {:?})",
                pending_req_infos.req_body, pending_req_infos.recipient_node
            );
        }
    }
    // Delete timeout requests
    for ws2p_req_id in requests_timeout {
        let _request_option = ws2p_module.requests_awaiting_response.remove(&ws2p_req_id);
    }
}

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

//! Define ws2p connections messages.

use super::*;
use durs_network_documents::NodeFullId;
use ws::Message;

#[derive(Debug)]
/// WS2Pv1 connection Message
pub struct WS2PConnectionMessage(pub NodeFullId, pub WS2PConnectionMessagePayload);

#[derive(Debug)]
/// WS2Pv1 connection Message payload
pub enum WS2PConnectionMessagePayload {
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
    DalRequest(ModuleReqId, serde_json::Value),
    PeerCard(serde_json::Value, Vec<EndpointV1>),
    Heads(Vec<serde_json::Value>),
    Document(BlockchainDocument),
    ReqResponse(ModuleReqId, serde_json::Value),
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

pub fn ws2p_conn_message_pretreatment(
    ws2p_module: &mut WS2PModule,
    message: WS2PConnectionMessage,
) -> WS2PSignal {
    let ws2p_full_id = message.0;
    match message.1 {
        WS2PConnectionMessagePayload::WrongUrl
        | WS2PConnectionMessagePayload::FailOpenWS
        | WS2PConnectionMessagePayload::FailToSplitWS => {
            let dal_ep = ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !");
            dal_ep.state = WS2PConnectionState::WSError;
            dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
            return WS2PSignal::WSError(ws2p_full_id);
        }
        WS2PConnectionMessagePayload::TryToSendConnectMess => {
            ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !")
                .state = WS2PConnectionState::TryToSendConnectMess;
        }
        WS2PConnectionMessagePayload::FailSendConnectMess => {
            let dal_ep = ws2p_module
                .ws2p_endpoints
                .get_mut(&ws2p_full_id)
                .expect("WS2P: Fail to get mut ep !");
            dal_ep.state = WS2PConnectionState::Unreachable;
            dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
        }
        WS2PConnectionMessagePayload::WebsocketOk(sender) => {
            ws2p_module.websockets.insert(ws2p_full_id, sender);
        }
        WS2PConnectionMessagePayload::ValidConnectMessage(response, new_con_state) => {
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
        WS2PConnectionMessagePayload::ValidAckMessage(response, new_con_state) => {
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
        WS2PConnectionMessagePayload::ValidOk(new_con_state) => {
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
        WS2PConnectionMessagePayload::DalRequest(req_id, req_body) => {
            return WS2PSignal::DalRequest(ws2p_full_id, req_id, req_body);
        }
        WS2PConnectionMessagePayload::PeerCard(body, ws2p_endpoints) => {
            return WS2PSignal::PeerCard(ws2p_full_id, body, ws2p_endpoints);
        }
        WS2PConnectionMessagePayload::Heads(heads) => {
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
        WS2PConnectionMessagePayload::Document(network_doc) => {
            return WS2PSignal::Document(ws2p_full_id, network_doc);
        }
        WS2PConnectionMessagePayload::ReqResponse(req_id, response) => {
            if ws2p_module.requests_awaiting_response.len() > req_id.0 as usize {
                if let Some((ref ws2p_request, ref recipient_fulld_id, ref _timestamp)) =
                    ws2p_module.requests_awaiting_response.remove(&req_id)
                {
                    return WS2PSignal::ReqResponse(
                        req_id,
                        *ws2p_request,
                        *recipient_fulld_id,
                        response,
                    );
                }
            }
        }
        WS2PConnectionMessagePayload::NegociationTimeout => {
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
        WS2PConnectionMessagePayload::Timeout => {
            close_connection(
                ws2p_module,
                &ws2p_full_id,
                WS2PCloseConnectionReason::Timeout,
            );
            return WS2PSignal::Timeout(ws2p_full_id);
        }
        WS2PConnectionMessagePayload::UnknowMessage => {
            warn!("WS2P : Receive Unknow Message from {}.", &ws2p_full_id.1)
        }
        WS2PConnectionMessagePayload::WrongFormatMessage => warn!(
            "WS2P : Receive Wrong Format Message from {}.",
            &ws2p_full_id.1
        ),
        WS2PConnectionMessagePayload::InvalidMessage => return WS2PSignal::Empty,
        WS2PConnectionMessagePayload::Close => close_connection(
            ws2p_module,
            &ws2p_full_id,
            WS2PCloseConnectionReason::AuthMessInvalidSig,
        ),
    }
    let connections_count = ws2p_module.websockets.len();
    if connections_count == 0 {
        return WS2PSignal::NoConnection;
    }
    // Detect timeout requests
    let mut requests_timeout = Vec::new();
    for &(ref req, ref ws2p_full_id, ref timestamp) in
        ws2p_module.requests_awaiting_response.clone().values()
    {
        if unwrap!(SystemTime::now().duration_since(*timestamp)) > Duration::new(20, 0) {
            requests_timeout.push(req.get_req_full_id());
            warn!("request timeout : {:?} (sent to {:?})", req, ws2p_full_id);
        }
    }
    // Delete (and resend) timeout requests
    for req_id in requests_timeout {
        //let ws2p_endpoints = ws2p_module.ws2p_endpoints.clone();
        let _request_option = ws2p_module.requests_awaiting_response.remove(&req_id.1);
        /*if let Some((request, _, _)) = request_option {
            let _request_result = ws2p_module.send_request_to_specific_node(
                &get_random_connection(&ws2p_endpoints),
                &request,
            );
        }*/
    }
    WS2PSignal::Empty
}

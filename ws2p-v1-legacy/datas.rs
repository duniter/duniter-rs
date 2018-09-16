//  Copyright (C) 2018  The Duniter Project Developers.
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

use constants::*;
use duniter_crypto::keys::*;
use duniter_dal::dal_requests::DALRequest;
use duniter_documents::Blockstamp;
use duniter_message::DuniterMessage;
use duniter_network::network_endpoint::*;
use duniter_network::network_head::*;
use duniter_network::*;
use std::collections::HashSet;
use std::sync::mpsc;
use *;

#[derive(Debug)]
pub struct WS2PModuleDatas {
    pub followers: Vec<mpsc::Sender<DuniterMessage>>,
    pub currency: Option<String>,
    pub key_pair: Option<KeyPairEnum>,
    pub conf: WS2PConf,
    pub node_id: NodeId,
    pub main_thread_channel: (
        mpsc::Sender<WS2PThreadSignal>,
        mpsc::Receiver<WS2PThreadSignal>,
    ),
    pub ws2p_endpoints: HashMap<NodeFullId, (EndpointEnum, WS2PConnectionState)>,
    pub websockets: HashMap<NodeFullId, WsSender>,
    pub requests_awaiting_response: HashMap<ModuleReqId, (NetworkRequest, NodeFullId, SystemTime)>,
    pub heads_cache: HashMap<NodeFullId, NetworkHead>,
    pub my_head: Option<NetworkHead>,
    pub uids_cache: HashMap<PubKey, String>,
}

impl WS2PModuleDatas {
    pub fn open_db(db_path: &PathBuf) -> Result<sqlite::Connection, sqlite::Error> {
        let conn: sqlite::Connection;
        if !db_path.as_path().exists() {
            conn = sqlite::open(db_path.as_path())?;
            conn.execute(
                "CREATE TABLE endpoints (hash_full_id TEXT, status INTEGER, node_id INTEGER, pubkey TEXT,
                api INTEGER, version INTEGER, endpoint TEXT, last_check INTEGER);",
            )?;
        } else {
            conn = sqlite::open(db_path.as_path())?;
        }
        Ok(conn)
    }
    pub fn send_dal_request(&self, req: &DALRequest) {
        for follower in &self.followers {
            if follower
                .send(DuniterMessage::DALRequest(req.clone()))
                .is_err()
            {
                // handle error
            }
        }
    }
    pub fn send_network_event(&self, event: &NetworkEvent) {
        for follower in &self.followers {
            match follower.send(DuniterMessage::NetworkEvent(event.clone())) {
                Ok(_) => {
                    debug!("Send NetworkEvent to one follower.");
                }
                Err(_) => {
                    warn!("Fail to send NetworkEvent to one follower !");
                }
            }
        }
    }
    pub fn get_network_consensus(&self) -> Result<Blockstamp, NetworkConsensusError> {
        let mut count_known_blockstamps = 0;
        let mut farthest_blockstamp = Blockstamp::default();
        let mut blockstamps_occurences: HashMap<Blockstamp, usize> =
            HashMap::with_capacity(*WS2P_DEFAULT_OUTCOMING_QUOTA);
        let mut dominant_blockstamp = Blockstamp::default();
        let mut dominant_blockstamp_occurences = 0;
        for (_ws2p_full_id, head) in self.heads_cache.clone() {
            count_known_blockstamps += 1;
            let blockstamps_occurences_copy = blockstamps_occurences.clone();
            match blockstamps_occurences_copy.get(&head.blockstamp()) {
                Some(occurences) => {
                    let mut occurences_mut = blockstamps_occurences
                        .get_mut(&head.blockstamp())
                        .expect("WS2P: Fail to get_mut blockstamps_occurences !");
                    *occurences_mut += 1;
                    if *occurences > dominant_blockstamp_occurences {
                        dominant_blockstamp_occurences = *occurences;
                        dominant_blockstamp = head.blockstamp();
                    }
                }
                None => {
                    blockstamps_occurences.insert(head.blockstamp(), 0);
                }
            }
            if head.blockstamp().id.0 > farthest_blockstamp.id.0 {
                farthest_blockstamp = head.blockstamp();
            }
        }
        if count_known_blockstamps < 5 {
            return Err(NetworkConsensusError::InsufficientData(
                count_known_blockstamps,
            ));
        } else if farthest_blockstamp == dominant_blockstamp {
            return Ok(dominant_blockstamp);
        }
        Err(NetworkConsensusError::Fork())
    }
    fn count_established_connections(&self) -> usize {
        let mut count_established_connections = 0;
        for (_ws2p_full_id, (_ep, state)) in self.ws2p_endpoints.clone() {
            if let WS2PConnectionState::Established = state {
                count_established_connections += 1;
            }
        }
        count_established_connections
    }
    pub fn connect_to_know_endpoints(&mut self) -> () {
        info!("WS2P: connect to know endpoints...");
        let mut count_established_connections = 0;
        let mut pubkeys = HashSet::new();
        let mut reachable_endpoints = Vec::new();
        let mut unreachable_endpoints = Vec::new();
        for (_ws2p_full_id, (ep, state)) in self.ws2p_endpoints.clone() {
            if ep.pubkey() == self.key_pair.unwrap().public_key() || !pubkeys.contains(&ep.pubkey())
            {
                match state {
                    WS2PConnectionState::Established => count_established_connections += 1,
                    WS2PConnectionState::NeverTry
                    | WS2PConnectionState::Close
                    | WS2PConnectionState::Denial => {
                        pubkeys.insert(ep.pubkey());
                        reachable_endpoints.push(ep);
                    }
                    _ => {
                        pubkeys.insert(ep.pubkey());
                        unreachable_endpoints.push(ep);
                    }
                }
            }
        }
        let mut free_outcoming_rooms =
            self.conf.clone().outcoming_quota - count_established_connections;
        while free_outcoming_rooms > 0 {
            let ep = if !reachable_endpoints.is_empty() {
                reachable_endpoints
                    .pop()
                    .expect("WS2P: Fail to pop() reachable_endpoints !")
            } else if !unreachable_endpoints.is_empty() {
                unreachable_endpoints
                    .pop()
                    .expect("WS2P: Fail to pop() unreachable_endpoints !")
            } else {
                break;
            };
            if cfg!(feature = "ssl") || ep.port() != 443 {
                self.connect_to_without_checking_quotas(&ep);
                free_outcoming_rooms -= 1;
            }
        }
    }
    pub fn connect_to(&mut self, endpoint: &EndpointEnum) -> () {
        // Add endpoint to endpoints list (if there isn't already)
        match self.ws2p_endpoints.get(
            &endpoint
                .node_full_id()
                .expect("WS2P: Fail to get ep.node_full_id() !"),
        ) {
            Some(_) => {
                self.ws2p_endpoints
                    .get_mut(
                        &endpoint
                            .node_full_id()
                            .expect("WS2P: Fail to get ep.node_full_id() !"),
                    ).expect("WS2P: Fail to get_mut() a ws2p_endpoint !")
                    .1 = WS2PConnectionState::NeverTry;
            }
            None => {
                self.ws2p_endpoints.insert(
                    endpoint
                        .node_full_id()
                        .expect("WS2P: Fail to get ep.node_full_id() !"),
                    (endpoint.clone(), WS2PConnectionState::NeverTry),
                );
            }
        };
        if self.conf.clone().outcoming_quota > self.count_established_connections() {
            self.connect_to_without_checking_quotas(&endpoint);
        }
    }
    fn close_connection(&mut self, ws2p_full_id: &NodeFullId, reason: WS2PCloseConnectionReason) {
        match reason {
            WS2PCloseConnectionReason::NegociationTimeout => {}
            WS2PCloseConnectionReason::AuthMessInvalidSig
            | WS2PCloseConnectionReason::Timeout
            | WS2PCloseConnectionReason::Unknow => {
                self.ws2p_endpoints
                    .get_mut(ws2p_full_id)
                    .expect("Failure : attempt to delete a non-existent connection !")
                    .1 = WS2PConnectionState::Close
            }
        }
        if let Some(websocket) = self.websockets.get(&ws2p_full_id) {
            let _result = websocket.0.close(ws::CloseCode::Normal);
        }
        let _result = self.websockets.remove(ws2p_full_id);
    }
    pub fn ws2p_conn_message_pretreatment(&mut self, message: WS2PConnectionMessage) -> WS2PSignal {
        let ws2p_full_id = message.0;
        match message.1 {
            WS2PConnectionMessagePayload::WrongUrl
            | WS2PConnectionMessagePayload::FailOpenWS
            | WS2PConnectionMessagePayload::FailToSplitWS => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !")
                    .1 = WS2PConnectionState::WSError;
                return WS2PSignal::WSError(ws2p_full_id);
            }
            WS2PConnectionMessagePayload::TryToSendConnectMess => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !")
                    .1 = WS2PConnectionState::TryToSendConnectMess;
            }
            WS2PConnectionMessagePayload::FailSendConnectMess => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to mut ep !")
                    .1 = WS2PConnectionState::Unreachable;
            }
            WS2PConnectionMessagePayload::WebsocketOk(sender) => {
                self.websockets.insert(ws2p_full_id, sender);
            }
            WS2PConnectionMessagePayload::ValidConnectMessage(response, new_con_state) => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !")
                    .1 = new_con_state;
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("Endpoint don't exist !")
                    .1 = WS2PConnectionState::ConnectMessOk;
                debug!("Send: {:#?}", response);
                self.websockets
                    .get_mut(&ws2p_full_id)
                    .unwrap_or_else(|| panic!("Fatal error : no websocket for {} !", ws2p_full_id))
                    .0
                    .send(Message::text(response))
                    .expect("WS2P: Fail to send OK Message !");
            }
            WS2PConnectionMessagePayload::ValidAckMessage(r, new_con_state) => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !")
                    .1 = new_con_state;
                if let WS2PConnectionState::AckMessOk = self.ws2p_endpoints[&ws2p_full_id].1 {
                    trace!("DEBUG : Send: {:#?}", r);
                    self.websockets
                        .get_mut(&ws2p_full_id)
                        .unwrap_or_else(|| {
                            panic!("Fatal error : no websocket for {} !", ws2p_full_id)
                        }).0
                        .send(Message::text(r))
                        .expect("WS2P: Fail to send Message in websocket !");
                }
            }
            WS2PConnectionMessagePayload::ValidOk(new_con_state) => {
                self.ws2p_endpoints
                    .get_mut(&ws2p_full_id)
                    .expect("WS2P: Fail to get mut ep !")
                    .1 = new_con_state;
                match self.ws2p_endpoints[&ws2p_full_id].1 {
                    WS2PConnectionState::OkMessOkWaitingAckMess => {}
                    WS2PConnectionState::Established => {
                        return WS2PSignal::ConnectionEstablished(ws2p_full_id)
                    }
                    _ => {
                        self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::Unknow);
                        return WS2PSignal::Empty;
                    }
                }
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
                            && (self.my_head.is_none() || head.node_full_id() != self
                                .my_head
                                .clone()
                                .expect("WS2P: Fail to clone my_head")
                                .node_full_id())
                            && head.apply(&mut self.heads_cache)
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
                if self.requests_awaiting_response.len() > req_id.0 as usize {
                    if let Some((ref ws2p_request, ref recipient_fulld_id, ref _timestamp)) =
                        self.requests_awaiting_response.remove(&req_id)
                    {
                        return WS2PSignal::ReqResponse(
                            req_id,
                            ws2p_request.clone(),
                            *recipient_fulld_id,
                            response,
                        );
                    }
                }
            }
            WS2PConnectionMessagePayload::NegociationTimeout => {
                match self.ws2p_endpoints[&ws2p_full_id].1 {
                    WS2PConnectionState::AckMessOk | WS2PConnectionState::ConnectMessOk => {
                        self.ws2p_endpoints
                            .get_mut(&ws2p_full_id)
                            .expect("WS2P: Fail to get mut ep !")
                            .1 = WS2PConnectionState::Denial
                    }
                    WS2PConnectionState::WaitingConnectMess => {
                        self.ws2p_endpoints
                            .get_mut(&ws2p_full_id)
                            .expect("WS2P: Fail to get mut ep !")
                            .1 = WS2PConnectionState::NoResponse
                    }
                    _ => {
                        self.ws2p_endpoints
                            .get_mut(&ws2p_full_id)
                            .expect("WS2P: Fail to get mut ep !")
                            .1 = WS2PConnectionState::Unreachable
                    }
                }
                self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::NegociationTimeout);
                return WS2PSignal::NegociationTimeout(ws2p_full_id);
            }
            WS2PConnectionMessagePayload::Timeout => {
                self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::Timeout);
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
            WS2PConnectionMessagePayload::Close => {
                if self.websockets.contains_key(&ws2p_full_id) {
                    self.close_connection(
                        &ws2p_full_id,
                        WS2PCloseConnectionReason::AuthMessInvalidSig,
                    )
                }
            }
        }
        let connections_count = self.websockets.len();
        if connections_count == 0 {
            return WS2PSignal::NoConnection;
        }
        // Detect timeout requests
        let mut requests_timeout = Vec::new();
        for &(ref req, ref _ws2p_full_id, ref timestamp) in
            self.requests_awaiting_response.clone().values()
        {
            if SystemTime::now().duration_since(*timestamp).unwrap() > Duration::new(20, 0) {
                requests_timeout.push(req.get_req_full_id());
                warn!("request timeout : {:?}", req);
            }
        }
        // Delete (and resend) timeout requests
        for req_id in requests_timeout {
            //let ws2p_endpoints = self.ws2p_endpoints.clone();
            let _request_option = self.requests_awaiting_response.remove(&req_id.1);
            /*if let Some((request, _, _)) = request_option {
                let _request_result = self.send_request_to_specific_node(
                    &get_random_connection(&ws2p_endpoints),
                    &request,
                );
            }*/
        }
        WS2PSignal::Empty
    }

    /*pub fn send_request_to_all_connections(
        &mut self,
        ws2p_request: &NetworkRequest,
    ) -> Result<(), SendRequestError> {
        let mut count_successful_sending: usize = 0;
        let mut errors: Vec<ws::Error> = Vec::new();
        match *ws2p_request {
            NetworkRequest::GetCurrent(req_full_id, _receiver) => {
                for (ws2p_full_id, (_ep, state)) in self.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        let ws2p_request = NetworkRequest::GetCurrent(
                            ModuleReqFullId(
                                req_full_id.0,
                                ModuleReqId(
                                    (self.requests_awaiting_response.len()
                                        + count_successful_sending)
                                        as u32,
                                ),
                            ),
                            ws2p_full_id,
                        );
                        match self.send_request_to_specific_node(&ws2p_full_id, &ws2p_request) {
                            Ok(_) => count_successful_sending += 1,
                            Err(e) => errors.push(e),
                        };
                    }
                }
            }
            /* NetworkRequest::GetBlock(req_full_id, number) => {} */
            NetworkRequest::GetBlocks(_req_full_id, _receiver, _count, _from_number) => {}
            NetworkRequest::GetRequirementsPending(req_full_id, _receiver, min_cert) => {
                for (ws2p_full_id, (_ep, state)) in self.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        let ws2p_request = NetworkRequest::GetRequirementsPending(
                            ModuleReqFullId(
                                req_full_id.0,
                                ModuleReqId(self.requests_awaiting_response.len() as u32),
                            ),
                            ws2p_full_id,
                            min_cert,
                        );
                        match self.send_request_to_specific_node(&ws2p_full_id, &ws2p_request) {
                            Ok(_) => count_successful_sending += 1,
                            Err(e) => errors.push(e),
                        };
                    }
                }
            }
            _ => {
                return Err(SendRequestError::RequestTypeMustNotBeTransmitted());
            }
        }
        debug!("count_successful_sending = {}", count_successful_sending);
        if !errors.is_empty() {
            return Err(SendRequestError::WSError(count_successful_sending, errors));
        }
        Ok(())
    }*/

    pub fn send_request_to_specific_node(
        &mut self,
        receiver_ws2p_full_id: &NodeFullId,
        ws2p_request: &NetworkRequest,
    ) -> ws::Result<()> {
        self.websockets
            .get_mut(receiver_ws2p_full_id)
            .expect("WS2P: Fail to get mut websocket !")
            .0
            .send(Message::text(
                network_request_to_json(ws2p_request).to_string(),
            ))?;
        self.requests_awaiting_response.insert(
            ws2p_request.get_req_id(),
            (
                ws2p_request.clone(),
                *receiver_ws2p_full_id,
                SystemTime::now(),
            ),
        );
        debug!(
            "send request {} to {}",
            network_request_to_json(ws2p_request).to_string(),
            receiver_ws2p_full_id
        );
        Ok(())
    }

    fn connect_to_without_checking_quotas(&mut self, endpoint: &EndpointEnum) {
        let endpoint_copy = endpoint.clone();
        let conductor_sender_copy = self.main_thread_channel.0.clone();
        let currency_copy = self.currency.clone();
        let key_pair_copy = self.key_pair;
        thread::spawn(move || {
            let _result = connect_to_ws2p_endpoint(
                &endpoint_copy,
                &conductor_sender_copy,
                &currency_copy.expect("WS2PError : No currency !"),
                key_pair_copy.expect("WS2PError : No key_pair !"),
            );
        });
    }
}

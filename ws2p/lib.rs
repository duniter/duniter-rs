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

//! Crate containing Duniter-rust core.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_debug_implementations, missing_copy_implementations, trivial_casts, unsafe_code,
    unstable_features, unused_import_braces, unused_qualifications
)]
#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_json;

extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate rand;
extern crate sqlite;
extern crate websocket;

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use duniter_crypto::keys::ed25519::Signature;
use duniter_crypto::keys::{ed25519, KeyPair, PrivateKey, PublicKey};
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALReqBlockchain, DALRequest, DALResBlockchain, DALResponse};
use duniter_dal::parsers::blocks::parse_json_block;
use duniter_documents::blockchain::Document;
use duniter_documents::Blockstamp;
use duniter_message::DuniterMessage;
use duniter_module::*;
use duniter_network::network_endpoint::*;
use duniter_network::network_head::*;
use duniter_network::*;

use websocket::{ClientBuilder, Message};

mod ack_message;
mod connect_message;
pub mod constants;
mod ok_message;
pub mod ws2p_connection;
pub mod ws2p_db;
pub mod ws2p_requests;

use self::ack_message::WS2PAckMessageV1;
use self::connect_message::WS2PConnectMessageV1;
use self::constants::*;
use self::ok_message::WS2POkMessageV1;
use self::rand::Rng;
use self::ws2p_connection::*;
use self::ws2p_requests::network_request_to_json;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WS2PConf {
    pub node_id: NodeUUID,
    pub outcoming_quota: usize,
    pub sync_endpoints: Vec<NetworkEndpoint>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WS2PConfParseError {
    UnknowError(),
}

#[derive(Debug)]
pub enum WS2PSignal {
    WSError(NodeFullId),
    ConnectionEstablished(NodeFullId),
    NegociationTimeout(NodeFullId),
    Timeout(NodeFullId),
    DalRequest(NodeFullId, ModuleReqId, serde_json::Value),
    PeerCard(NodeFullId, serde_json::Value, Vec<NetworkEndpoint>),
    Heads(NodeFullId, Vec<NetworkHead>),
    Document(NodeFullId, NetworkDocument),
    ReqResponse(ModuleReqId, NetworkRequest, NodeFullId, serde_json::Value),
    Empty,
    NoConnection,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NetworkConsensusError {
    InsufficientData(usize),
    Fork,
}

#[derive(Debug)]
pub enum SendRequestError {
    RequestTypeMustNotBeTransmitted(),
    WSError(usize, Vec<websocket::WebSocketError>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WS2PModule {}

#[derive(Debug)]
pub struct WS2PModuleDatas {
    pub followers: Vec<mpsc::Sender<DuniterMessage>>,
    pub currency: Option<String>,
    pub key_pair: Option<ed25519::KeyPair>,
    pub conf: Option<WS2PConf>,
    pub main_thread_channel: (
        mpsc::Sender<WS2PThreadSignal>,
        mpsc::Receiver<WS2PThreadSignal>,
    ),
    pub ws2p_endpoints: HashMap<NodeFullId, (NetworkEndpoint, WS2PConnectionState)>,
    pub connections_meta_datas: HashMap<NodeFullId, WS2PConnectionMetaData>,
    pub websockets: HashMap<NodeFullId, WebsocketSender>,
    pub threads_senders_channels: HashMap<NodeFullId, mpsc::Sender<WS2POrderForListeningThread>>,
    pub requests_awaiting_response: HashMap<ModuleReqId, (NetworkRequest, NodeFullId, SystemTime)>,
    pub heads_cache: HashMap<NodeFullId, NetworkHead>,
    pub my_head: Option<NetworkHead>,
    pub uids_cache: HashMap<ed25519::PublicKey, String>,
}

#[derive(Debug)]
pub enum WS2PThreadSignal {
    DuniterMessage(DuniterMessage),
    WS2PConnectionMessage(WS2PConnectionMessage),
}

pub trait WS2PMessage: Sized {
    fn parse(v: &serde_json::Value, currency: String) -> Option<Self>;
    fn to_raw(&self) -> String;
    fn sign(&self, key_pair: ed25519::KeyPair) -> Signature {
        key_pair.sign(self.to_raw().as_bytes())
    }
    fn verify(&self) -> bool;
    //fn parse_and_verify(v: serde_json::Value, currency: String) -> bool;
}

pub fn get_random_connection(
    connections: &HashMap<NodeFullId, (NetworkEndpoint, WS2PConnectionState)>,
) -> NodeFullId {
    let mut rng = rand::thread_rng();
    let mut loop_count = 0;
    loop {
        for (ws2p_full_id, (_ep, state)) in connections.clone() {
            if loop_count > 10 {
                return ws2p_full_id;
            }
            if let WS2PConnectionState::Established = state {
                if rng.gen::<bool>() {
                    return ws2p_full_id;
                }
            }
        }
        loop_count += 1;
    }
}

impl Default for WS2PModule {
    fn default() -> WS2PModule {
        WS2PModule {}
    }
}

impl DuniterModule<ed25519::KeyPair, DuniterMessage> for WS2PModule {
    fn id() -> ModuleId {
        ModuleId::Str("ws2p")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Essential()
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::NetworkKeyPair()
    }
    fn default_conf() -> serde_json::Value {
        json!({
            "sync_peers": [{
                "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef",
                "ws2p_endpoints": ["WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p"]
            }]
        })
    }
    fn start(
        soft_name: &str,
        soft_version: &str,
        keys: RequiredKeysContent<ed25519::KeyPair>,
        duniter_conf: &DuniterConf,
        module_conf: &serde_json::Value,
        rooter_sender: mpsc::Sender<RooterThreadMessage<DuniterMessage>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError> {
        let start_time = SystemTime::now();
        let mut ws2p_module = WS2PModuleDatas {
            followers: Vec::new(),
            key_pair: None,
            currency: None,
            conf: None,
            main_thread_channel: mpsc::channel(),
            ws2p_endpoints: HashMap::new(),
            connections_meta_datas: HashMap::new(),
            websockets: HashMap::new(),
            threads_senders_channels: HashMap::new(),
            requests_awaiting_response: HashMap::new(),
            heads_cache: HashMap::new(),
            my_head: None,
            uids_cache: HashMap::new(),
        };

        // load conf
        let key_pair = match keys {
            RequiredKeysContent::NetworkKeyPair(key_pair) => key_pair,
            _ => panic!("WS2PModule fatal error at load_conf() : keys != NetworkKeyPair"),
        };
        let conf = WS2PModuleDatas::parse_ws2p_conf(duniter_conf, module_conf);
        let mut ws2p_endpoints = HashMap::new();
        for ep in conf.sync_endpoints.clone() {
            ws2p_endpoints.insert(
                ep.node_full_id().unwrap(),
                (ep.clone(), WS2PConnectionState::Close),
            );
            info!("Load sync endpoint {}", ep.raw());
        }
        ws2p_module.key_pair = Some(key_pair.clone());
        ws2p_module.currency = Some(duniter_conf.currency().to_string());
        ws2p_module.conf = Some(conf.clone());
        ws2p_module.ws2p_endpoints = ws2p_endpoints;

        // Create ws2p main thread channel
        let ws2p_sender_clone = ws2p_module.main_thread_channel.0.clone();

        // Create proxy channel
        let (proxy_sender, proxy_receiver): (
            mpsc::Sender<DuniterMessage>,
            mpsc::Receiver<DuniterMessage>,
        ) = mpsc::channel();
        let proxy_sender_clone = proxy_sender.clone();

        // Launch a proxy thread that transform DuniterMessage to WS2PThreadSignal(DuniterMessage)
        thread::spawn(move || {
            // Send proxy sender to main
            match rooter_sender.send(RooterThreadMessage::ModuleSender(proxy_sender_clone)) {
                Ok(_) => {
                    debug!("Send ws2p sender to main thread.");
                }
                Err(_) => panic!("Fatal error : ws2p module fail to send is sender channel !"),
            }
            //drop(rooter_sender);
            loop {
                match proxy_receiver.recv() {
                    Ok(message) => match ws2p_sender_clone
                        .send(WS2PThreadSignal::DuniterMessage(message.clone()))
                    {
                        Ok(_) => {
                            if let DuniterMessage::Stop() = message {
                                break;
                            };
                        }
                        Err(_) => panic!(
                            "Fatal error : fail to relay DuniterMessage to ws2p main thread !"
                        ),
                    },
                    Err(e) => panic!(format!("{}", e)),
                }
            }
        });

        // open ws2p bdd
        let mut db_path =
            duniter_conf::datas_path(duniter_conf.profile().as_str(), &duniter_conf.currency());
        db_path.push("ws2p.db");
        let db = WS2PModuleDatas::open_db(db_path).expect("Fatal error : fail to open WS2P DB !");

        // Get ws2p endpoints in BDD
        let mut count = 0;
        let dal_enpoints =
            ws2p_db::get_endpoints_for_api(&db, NetworkEndpointApi(String::from("WS2P")));
        for ep in dal_enpoints {
            if ep.api() == NetworkEndpointApi(String::from("WS2P")) && ep.port() != 443 {
                count += 1;
                ws2p_module.ws2p_endpoints.insert(
                    ep.node_full_id().unwrap(),
                    (ep.clone(), WS2PConnectionState::from(ep.status())),
                );
            }
        }
        info!("Load {} endpoints from bdd !", count);

        // Stop here in load_conf_only mode
        if load_conf_only {
            return Ok(());
        }

        // Initialize variables
        let mut last_ws2p_connecting_wave = SystemTime::now();
        let mut last_ws2p_connections_print = SystemTime::now();
        let mut endpoints_to_update_status: HashMap<NodeFullId, SystemTime> = HashMap::new();
        let mut last_identities_request = UNIX_EPOCH;
        let mut current_blockstamp = Blockstamp::default();
        let mut next_receiver = 0;

        // Start
        ws2p_module.connect_to_know_endpoints();
        loop {
            match ws2p_module
                .main_thread_channel
                .1
                .recv_timeout(Duration::from_millis(200))
            {
                Ok(message) => match message {
                    WS2PThreadSignal::DuniterMessage(ref duniter_mesage) => {
                        match duniter_mesage {
                            &DuniterMessage::Stop() => break,
                            &DuniterMessage::Followers(ref new_followers) => {
                                info!("WS2P module receive followers !");
                                for new_follower in new_followers {
                                    debug!("WS2PModule : push one follower.");
                                    ws2p_module.followers.push(new_follower.clone());
                                    if current_blockstamp == Blockstamp::default() {
                                        // Request local current blockstamp
                                        ws2p_module.send_dal_request(
                                            &DALRequest::BlockchainRequest(
                                                DALReqBlockchain::CurrentBlock(ModuleReqFullId(
                                                    WS2PModule::id(),
                                                    ModuleReqId(0),
                                                )),
                                            ),
                                        );
                                    } else {
                                        if ws2p_module.my_head.is_none() {
                                            ws2p_module.my_head =
                                                Some(WS2PModuleDatas::generate_my_head(
                                                    &key_pair.clone(),
                                                    &conf.clone(),
                                                    soft_name,
                                                    soft_version,
                                                    &current_blockstamp,
                                                    None,
                                                ));
                                        }
                                        ws2p_module.send_network_event(
                                            &NetworkEvent::ReceiveHeads(vec![
                                                ws2p_module.my_head.clone().unwrap(),
                                            ]),
                                        );
                                    }
                                }
                            }
                            &DuniterMessage::NetworkRequest(ref request) => match request {
                                &NetworkRequest::GetBlocks(
                                    ref req_id,
                                    ref receiver,
                                    ref count,
                                    ref from,
                                ) => {
                                    if *receiver == NodeFullId::default() {
                                        let mut receiver_index = 0;
                                        let mut real_receiver = NodeFullId::default();
                                        for (ws2p_full_id, (_ep, state)) in
                                            ws2p_module.ws2p_endpoints.clone()
                                        {
                                            if let WS2PConnectionState::Established = state {
                                                if receiver_index == next_receiver {
                                                    real_receiver = ws2p_full_id;
                                                    break;
                                                }
                                                receiver_index += 1;
                                            }
                                        }
                                        if real_receiver == NodeFullId::default() {
                                            next_receiver = 0;
                                            for (ws2p_full_id, (_ep, state)) in
                                                ws2p_module.ws2p_endpoints.clone()
                                            {
                                                if let WS2PConnectionState::Established = state {
                                                    real_receiver = ws2p_full_id;
                                                    break;
                                                }
                                            }
                                        } else {
                                            next_receiver += 1;
                                        }
                                        if real_receiver != NodeFullId::default() {
                                            let _blocks_request_result = ws2p_module
                                                .send_request_to_specific_node(
                                                    &real_receiver,
                                                    &NetworkRequest::GetBlocks(
                                                        req_id.clone(),
                                                        receiver.clone(),
                                                        count.clone(),
                                                        from.clone(),
                                                    ),
                                                );
                                        } else {
                                            warn!("WS2PModule : No WS2P connections !");
                                        }
                                    } else {
                                        let _blocks_request_result = ws2p_module
                                            .send_request_to_specific_node(
                                                &receiver,
                                                &NetworkRequest::GetBlocks(
                                                    req_id.clone(),
                                                    receiver.clone(),
                                                    count.clone(),
                                                    from.clone(),
                                                ),
                                            );
                                    }
                                }
                                _ => {}
                            },
                            &DuniterMessage::DALEvent(ref dal_event) => match dal_event {
                                &DALEvent::StackUpValidBlock(ref block) => {
                                    current_blockstamp = block.deref().blockstamp();
                                    debug!(
                                        "WS2PModule : current_blockstamp = {}",
                                        current_blockstamp
                                    );
                                    ws2p_module.my_head = Some(WS2PModuleDatas::generate_my_head(
                                        &key_pair.clone(),
                                        &conf.clone(),
                                        soft_name,
                                        soft_version,
                                        &current_blockstamp,
                                        None,
                                    ));
                                    ws2p_module.send_network_event(&NetworkEvent::ReceiveHeads(
                                        vec![ws2p_module.my_head.clone().unwrap()],
                                    ));
                                }
                                _ => {}
                            },
                            &DuniterMessage::DALResponse(ref dal_res) => match dal_res {
                                &DALResponse::Blockchain(ref dal_res_bc) => match dal_res_bc {
                                    &DALResBlockchain::CurrentBlock(
                                        ref _requester_full_id,
                                        ref current_block,
                                    ) => {
                                        debug!(
                                            "WS2PModule : receive DALResBc::CurrentBlock({})",
                                            current_block.blockstamp()
                                        );
                                        current_blockstamp = current_block.blockstamp();
                                        if ws2p_module.my_head.is_none() {
                                            ws2p_module.my_head =
                                                Some(WS2PModuleDatas::generate_my_head(
                                                    &key_pair.clone(),
                                                    &conf.clone(),
                                                    soft_name,
                                                    soft_version,
                                                    &current_blockstamp,
                                                    None,
                                                ));
                                        }
                                        ws2p_module.send_network_event(
                                            &NetworkEvent::ReceiveHeads(vec![
                                                ws2p_module.my_head.clone().unwrap(),
                                            ]),
                                        );
                                    }
                                    &DALResBlockchain::UIDs(ref uids) => {
                                        // Add uids to heads
                                        for (_, head) in ws2p_module.heads_cache.iter_mut() {
                                            if let Some(uid_option) = uids.get(&head.pubkey()) {
                                                if let &Some(ref uid) = uid_option {
                                                    head.set_uid(uid);
                                                    ws2p_module
                                                        .uids_cache
                                                        .insert(head.pubkey(), uid.to_string());
                                                } else {
                                                    ws2p_module.uids_cache.remove(&head.pubkey());
                                                }
                                            }
                                        }
                                        // Resent heads to other modules
                                        ws2p_module.send_network_event(
                                            &NetworkEvent::ReceiveHeads(
                                                ws2p_module
                                                    .heads_cache
                                                    .values()
                                                    .map(|h| h.clone())
                                                    .collect(),
                                            ),
                                        );
                                        // Resent to other modules connections that match receive uids
                                        for (node_full_id, (_ep, conn_state)) in
                                            ws2p_module.ws2p_endpoints.clone()
                                        {
                                            if let Some(uid_option) = uids.get(&node_full_id.1) {
                                                ws2p_module.send_network_event(
                                                    &NetworkEvent::ConnectionStateChange(
                                                        node_full_id,
                                                        conn_state as u32,
                                                        uid_option.clone(),
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    WS2PThreadSignal::WS2PConnectionMessage(ws2p_conn_message) => match ws2p_module
                        .ws2p_conn_message_pretreatment(ws2p_conn_message)
                    {
                        WS2PSignal::NoConnection => {
                            warn!("WS2PSignal::NoConnection");
                            last_ws2p_connecting_wave = SystemTime::now();
                            ws2p_module.connect_to_know_endpoints();
                        }
                        WS2PSignal::ConnectionEstablished(ws2p_full_id) => {
                            let req_id =
                                ModuleReqId(ws2p_module.requests_awaiting_response.len() as u32);
                            let module_id = WS2PModule::id();
                            let _current_request_result = ws2p_module
                                .send_request_to_specific_node(
                                    &ws2p_full_id,
                                    &NetworkRequest::GetCurrent(
                                        ModuleReqFullId(module_id, req_id),
                                        ws2p_full_id,
                                    ),
                                );
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Established as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                            ));
                        }
                        WS2PSignal::WSError(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::WSError as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                            ));
                        }
                        WS2PSignal::NegociationTimeout(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Denial as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                            ));
                        }
                        WS2PSignal::Timeout(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Close as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                            ));
                        }
                        WS2PSignal::PeerCard(_ws2p_full_id, _peer_card, ws2p_endpoints) => {
                            //trace!("WS2PSignal::PeerCard({})", ws2p_full_id);
                            //ws2p_module.send_network_event(NetworkEvent::ReceivePeers(_));
                            for ep in ws2p_endpoints {
                                if ep.port() != 443 {
                                    match ws2p_module
                                        .ws2p_endpoints
                                        .get(&ep.node_full_id().unwrap())
                                    {
                                        Some(_) => {}
                                        None => {
                                            if let Some(_api) =
                                                ws2p_db::string_to_api(&ep.api().0.clone())
                                            {
                                                endpoints_to_update_status.insert(
                                                    ep.node_full_id().unwrap(),
                                                    SystemTime::now(),
                                                );
                                            }
                                            ws2p_module.connect_to(ep);
                                        }
                                    };
                                }
                            }
                        }
                        WS2PSignal::Heads(ws2p_full_id, heads) => {
                            trace!("WS2PSignal::Heads({}, {:?})", ws2p_full_id, heads.len());
                            ws2p_module.send_dal_request(&DALRequest::BlockchainRequest(
                                DALReqBlockchain::UIDs(heads.iter().map(|h| h.pubkey()).collect()),
                            ));
                            ws2p_module.send_network_event(&NetworkEvent::ReceiveHeads(
                                heads
                                    .iter()
                                    .map(|head| {
                                        let mut new_head = head.clone();
                                        if let Some(uid) =
                                            ws2p_module.uids_cache.get(&head.pubkey())
                                        {
                                            new_head.set_uid(uid);
                                        }
                                        new_head
                                    })
                                    .collect(),
                            ));
                        }
                        WS2PSignal::Document(ws2p_full_id, network_doc) => {
                            trace!("WS2PSignal::Document({})", ws2p_full_id);
                            ws2p_module.send_network_event(&NetworkEvent::ReceiveDocuments(vec![
                                network_doc,
                            ]));
                        }
                        WS2PSignal::ReqResponse(req_id, req, recipient_full_id, response) => {
                            match req {
                                NetworkRequest::GetCurrent(ref _req_id, _receiver) => {
                                    info!("WS2PSignal::ReceiveCurrent({}, {:?})", req_id.0, req);
                                    if let Some(block) = parse_json_block(&response) {
                                        ws2p_module.send_network_event(&NetworkEvent::ReqResponse(
                                            Box::new(NetworkResponse::CurrentBlock(
                                                ModuleReqFullId(WS2PModule::id(), req_id),
                                                recipient_full_id,
                                                Box::new(block),
                                            )),
                                        ));
                                    }
                                    /*if let Some(block) = BlockV10::from_json_value(&response) {
                                        ws2p_module
                                            .connections_meta_datas
                                            .get_mut(&recipient_full_id)
                                            .unwrap()
                                            .current_blockstamp = Some((block.id, block.hash));
                                    }*/
                                }
                                NetworkRequest::GetBlocks(ref _req_id, _receiver, _count, from) => {
                                    info!("WS2PSignal::ReceiveChunk({}, {:?})", req_id.0, req);
                                    if response.is_array() {
                                        let mut chunk = Vec::new();
                                        for json_block in response.as_array().unwrap() {
                                            if let Some(block) = parse_json_block(json_block) {
                                                chunk.push(NetworkDocument::Block(block));
                                            } else {
                                                warn!("WS2PModule: Error : fail to parse one json block !");
                                            }
                                        }
                                        debug!("Send chunk to followers : {}", from);
                                        ws2p_module.send_network_event(
                                            &NetworkEvent::ReceiveDocuments(chunk),
                                        );
                                    }
                                }
                                NetworkRequest::GetRequirementsPending(
                                    _req_id,
                                    _receiver,
                                    min_cert,
                                ) => {
                                    info!(
                                        "WS2PSignal::ReceiveRequirementsPending({}, {})",
                                        req_id.0, min_cert
                                    );
                                    debug!("----------------------------------------");
                                    debug!("-      BEGIN IDENTITIES PENDING        -");
                                    debug!("----------------------------------------");
                                    debug!("{:#?}", response);
                                    debug!("----------------------------------------");
                                    debug!("-       END IDENTITIES PENDING         -");
                                    debug!("----------------------------------------");
                                }
                                _ => {}
                            }
                        }
                        WS2PSignal::Empty => {}
                        _ => {}
                    },
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Disconnected => {
                        panic!("Disconnected ws2p module !");
                    }
                    mpsc::RecvTimeoutError::Timeout => {}
                },
            }
            if SystemTime::now()
                .duration_since(last_ws2p_connections_print)
                .unwrap() > Duration::new(5, 0)
            {
                last_ws2p_connections_print = SystemTime::now();
                let mut connected_nodes = Vec::new();
                let mut denial_nodes = Vec::new();
                let mut disconnected_nodes = Vec::new();
                let mut unreachable_nodes = Vec::new();
                let mut ws_error_nodes = Vec::new();
                for (k, (_ep, state)) in ws2p_module.ws2p_endpoints.clone() {
                    match state {
                        WS2PConnectionState::NeverTry => {
                            //writeln!("Never try : {}", k);
                        }
                        WS2PConnectionState::TryToOpenWS => {}//writeln!("TryToOpenWS : {}", k),
                        WS2PConnectionState::WSError => {
                            ws_error_nodes.push(k);
                        }
                        WS2PConnectionState::TryToSendConnectMess => {
                            //writeln!("TryToSendConnectMess : {}", k)
                        }
                        WS2PConnectionState::Unreachable => {
                            unreachable_nodes.push(k);
                        }
                        WS2PConnectionState::WaitingConnectMess => {
                            //writeln!("WaitingConnectMess : {}", k)
                        }
                        WS2PConnectionState::NoResponse => {}//writeln!("NoResponse : {}", k),
                        WS2PConnectionState::AckMessOk
                        | WS2PConnectionState::ConnectMessOk
                        | WS2PConnectionState::OkMessOkWaitingAckMess => {
                            //writeln!("Ongoing negotiations : {}", k)
                        }
                        WS2PConnectionState::Denial => {
                            denial_nodes.push(k);
                        }
                        WS2PConnectionState::Established => {
                            connected_nodes.push(k);
                        }
                        WS2PConnectionState::Close => {
                            disconnected_nodes.push(k);
                        }
                    }
                }
                /*writeln!(
                                    "Connected with {} nodes. (Denial : {}, Disconnected : {}, Unreachable: {}, WSError : {})",
                                    connected_nodes.len(),
                                    denial_nodes.len(),
                                    disconnected_nodes.len(),
                                    unreachable_nodes.len(),
                                    ws_error_nodes.len()
                                );*/
                for _node in connected_nodes.clone() {
                    //writeln!("Connection established : {}", node);
                }
                for _node in denial_nodes {
                    //writeln!("Denial : {}", node);
                }
                for _node in disconnected_nodes {
                    //writeln!("Disconnected : {}", node);
                }
                for _node in unreachable_nodes {
                    //writeln!("Unreachable : {}", node);
                }
                // Print network consensus
                match ws2p_module.get_network_consensus() {
                    Ok(consensus_blockstamp) => {
                        /*writeln!(
                            "WS2PModule : get_network_consensus() = {:?}",
                            consensus_blockstamp
                        );*/

                        while current_blockstamp.id > consensus_blockstamp.id {
                            warn!("Need to revert !");
                        }
                    }
                    Err(e) => warn!("{:?}", e),
                }
                // Print current_blockstamp
                info!(
                    "WS2PModule : current_blockstamp() = {:?}",
                    current_blockstamp
                );
                // New WS2P connection wave
                if connected_nodes.len() < ws2p_module.conf.clone().unwrap().outcoming_quota
                    && (SystemTime::now()
                        .duration_since(last_ws2p_connecting_wave)
                        .unwrap()
                        > Duration::new(*WS2P_OUTCOMING_INTERVAL, 0)
                        || (SystemTime::now()
                            .duration_since(last_ws2p_connecting_wave)
                            .unwrap()
                            > Duration::new(*WS2P_OUTCOMING_INTERVAL_AT_STARTUP, 0)
                            && SystemTime::now().duration_since(start_time).unwrap()
                                < Duration::new(*WS2P_OUTCOMING_INTERVAL, 0)))
                {
                    last_ws2p_connecting_wave = SystemTime::now();
                    info!("Connected to know endpoints...");
                    ws2p_module.connect_to_know_endpoints();
                }
                /*// Request blocks from network
                if SystemTime::now()
                    .duration_since(last_blocks_request)
                    .unwrap() > Duration::new(*BLOCKS_REQUEST_INTERVAL, 0)
                    && SystemTime::now().duration_since(start_time).unwrap() > Duration::new(10, 0)
                {
                    let mut request_blocks_from = current_blockstamp.id.0;
                    if request_blocks_from > 0 {
                        request_blocks_from += 1;
                    }
                    info!("get chunks from all connections...");
                    let module_id = WS2PModule::id();
                    let _blocks_request_result =
                        ws2p_module.send_request_to_all_connections(&NetworkRequest::GetBlocks(
                            ModuleReqFullId(module_id, ModuleReqId(0 as u32)),
                            NodeFullId::default(),
                            50,
                            request_blocks_from,
                        ));
                    last_blocks_request = SystemTime::now();
                }*/
                // Request pending_identities from network
                if SystemTime::now()
                    .duration_since(last_identities_request)
                    .unwrap()
                    > Duration::new(*PENDING_IDENTITIES_REQUEST_INTERVAL, 0)
                    && SystemTime::now().duration_since(start_time).unwrap() > Duration::new(10, 0)
                {
                    /*info!("get pending_identities from all connections...");
                                    let _blocks_request_result = ws2p_module.send_request_to_all_connections(
                                        &NetworkRequest::GetRequirementsPending(ModuleReqId(0 as u32), 5),
                                    );*/
                    last_identities_request = SystemTime::now();
                }
                // Write pending endpoints
                for (ep_full_id, received_time) in endpoints_to_update_status.clone() {
                    if SystemTime::now().duration_since(received_time).unwrap()
                        > Duration::new(*DURATION_BEFORE_RECORDING_ENDPOINT, 0)
                    {
                        if let Some(&(ref ep, ref state)) =
                            ws2p_module.ws2p_endpoints.get(&ep_full_id)
                        {
                            /*let dal_endpoint = duniter_dal::endpoint::DALEndpoint::new(
                                                state.clone() as u32,
                                                ep.node_uuid().unwrap().0,
                                                ep.pubkey(),
                                                duniter_dal::endpoint::string_to_api(&ep.api().0).unwrap(),
                                                1,
                                                ep.to_string(),
                                                received_time.duration_since(UNIX_EPOCH).unwrap(),
                                            );*/
                            ws2p_db::write_endpoint(
                                &db,
                                &ep,
                                state.to_u32(),
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            );
                        }
                        endpoints_to_update_status.remove(&ep_full_id);
                    } else {
                        info!(
                            "Write {} endpoint in {} secs.",
                            ep_full_id,
                            *DURATION_BEFORE_RECORDING_ENDPOINT
                                - SystemTime::now()
                                    .duration_since(received_time)
                                    .unwrap()
                                    .as_secs()
                        );
                    }
                }
                // ..
            }
        }
        Ok(())
    }
}

impl WS2PModuleDatas {
    fn open_db(db_path: PathBuf) -> Result<sqlite::Connection, sqlite::Error> {
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
    pub fn parse_ws2p_conf(
        duniter_conf: &DuniterConf,
        ws2p_json_conf: &serde_json::Value,
    ) -> WS2PConf {
        let mut sync_endpoints = Vec::new();
        match ws2p_json_conf.get("sync_peers") {
            Some(peers) => {
                let array_peers = peers.as_array().expect("Conf: Fail to parse conf file !");
                for peer in array_peers {
                    let pubkey = match peer.get("pubkey") {
                        Some(pubkey) => {
                            PublicKey::from_base58(
                                pubkey
                                    .as_str()
                                    .expect("WS2PConf Error : fail to parse sync endpoint pubkey"),
                            ).expect("WS2PConf Error : fail to parse sync endpoint pubkey")
                        }
                        None => panic!(
                            "Fail to load ws2p conf : \
                             WrongFormat : not found pubkey field !"
                        ),
                    };
                    match peer.get("ws2p_endpoints") {
                        Some(endpoints) => {
                            let array_endpoints = endpoints
                                .as_array()
                                .expect("Conf: Fail to parse conf file !");
                            for endpoint in array_endpoints {
                                sync_endpoints.push(
                                    NetworkEndpoint::parse_from_raw(
                                        endpoint.as_str().unwrap(),
                                        pubkey,
                                        0,
                                        0,
                                    ).expect(&format!(
                                        "WS2PConf Error : fail to parse sync Endpoint = {:?}",
                                        endpoint.as_str().unwrap()
                                    )),
                                );
                            }
                        }
                        None => panic!(
                            "Fail to load conf : \
                             WrongFormat : not found ws2p_endpoints field !"
                        ),
                    };
                }
            }
            None => panic!(
                "Configuration Error : \
                 You must declare at least one node on which to synchronize !"
            ),
        };
        WS2PConf {
            outcoming_quota: *WS2P_DEFAULT_OUTCOMING_QUOTA,
            node_id: NodeUUID(duniter_conf.my_node_id()),
            sync_endpoints,
        }
    }
    pub fn send_dal_request(&self, req: &DALRequest) {
        for follower in &self.followers {
            match follower.send(DuniterMessage::DALRequest(req.clone())) {
                Ok(_) => {}
                Err(_) => {}
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
    pub fn generate_my_head(
        network_keypair: &ed25519::KeyPair,
        conf: &WS2PConf,
        soft_name: &str,
        soft_version: &str,
        my_current_blockstamp: &Blockstamp,
        my_uid: Option<String>,
    ) -> NetworkHead {
        let message = NetworkHeadMessage::V2(NetworkHeadMessageV2 {
            api: String::from("WS2POCA"),
            version: 1,
            pubkey: network_keypair.pubkey,
            blockstamp: my_current_blockstamp.clone(),
            node_uuid: conf.node_id,
            software: String::from(soft_name),
            soft_version: String::from(soft_version),
            prefix: 1,
            free_member_room: None,
            free_mirror_room: None,
        });
        let message_v2 = NetworkHeadMessage::V2(NetworkHeadMessageV2 {
            api: String::from("WS2POCA"),
            version: 2,
            pubkey: network_keypair.pubkey,
            blockstamp: my_current_blockstamp.clone(),
            node_uuid: conf.node_id,
            software: String::from(soft_name),
            soft_version: String::from(soft_version),
            prefix: 1,
            free_member_room: Some(0),
            free_mirror_room: Some(0),
        });
        NetworkHead::V2(Box::new(NetworkHeadV2 {
            message: message.clone(),
            sig: network_keypair.privkey.sign(message.to_string().as_bytes()),
            message_v2: message_v2.clone(),
            sig_v2: network_keypair
                .privkey
                .sign(message_v2.to_string().as_bytes()),
            step: 0,
            uid: my_uid,
        }))
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
                    let mut occurences_mut =
                        blockstamps_occurences.get_mut(&head.blockstamp()).unwrap();
                    *occurences_mut += 1;
                    if *occurences > dominant_blockstamp_occurences {
                        dominant_blockstamp_occurences = *occurences;
                        dominant_blockstamp = head.blockstamp().clone();
                    }
                }
                None => {
                    blockstamps_occurences.insert(head.blockstamp().clone(), 0);
                }
            }
            if head.blockstamp().id.0 > farthest_blockstamp.id.0 {
                farthest_blockstamp = head.blockstamp().clone();
            }
        }
        if count_known_blockstamps < 5 {
            return Err(NetworkConsensusError::InsufficientData(
                count_known_blockstamps,
            ));
        } else if farthest_blockstamp == dominant_blockstamp {
            return Ok(dominant_blockstamp);
        }
        Err(NetworkConsensusError::Fork)
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
        let mut count_established_connections = 0;
        let mut reachable_endpoints = Vec::new();
        let mut unreachable_endpoints = Vec::new();
        for (_ws2p_full_id, (ep, state)) in self.ws2p_endpoints.clone() {
            match state {
                WS2PConnectionState::Established => count_established_connections += 1,
                WS2PConnectionState::NeverTry
                | WS2PConnectionState::Close
                | WS2PConnectionState::Denial => reachable_endpoints.push(ep),
                _ => unreachable_endpoints.push(ep),
            }
        }
        let mut free_outcoming_rooms =
            self.conf.clone().unwrap().outcoming_quota - count_established_connections;
        while free_outcoming_rooms > 0 {
            let ep = if !reachable_endpoints.is_empty() {
                reachable_endpoints.pop().unwrap()
            } else if !unreachable_endpoints.is_empty() {
                unreachable_endpoints.pop().unwrap()
            } else {
                break;
            };
            self.connect_to_without_checking_quotas(ep);
            free_outcoming_rooms -= 1;
        }
    }
    pub fn connect_to(&mut self, endpoint: NetworkEndpoint) -> () {
        // Add endpoint to endpoints list (if there isn't already)
        match self.ws2p_endpoints.get(&endpoint.node_full_id().unwrap()) {
            Some(_) => {
                self.ws2p_endpoints
                    .get_mut(&endpoint.node_full_id().unwrap())
                    .unwrap()
                    .1 = WS2PConnectionState::NeverTry;
            }
            None => {
                self.ws2p_endpoints.insert(
                    endpoint.node_full_id().unwrap(),
                    (endpoint.clone(), WS2PConnectionState::NeverTry),
                );
            }
        };
        if self.conf.clone().unwrap().outcoming_quota > self.count_established_connections() {
            self.connect_to_without_checking_quotas(endpoint);
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
        self.connections_meta_datas.remove(ws2p_full_id);
        self.websockets.remove(ws2p_full_id).expect(&format!(
            "Fatal error : no websocket for {} !",
            ws2p_full_id
        ));
        self.threads_senders_channels.remove(ws2p_full_id);
    }
    pub fn ws2p_conn_message_pretreatment(&mut self, message: WS2PConnectionMessage) -> WS2PSignal {
        let connections_count = self.connections_meta_datas.len();
        if connections_count == 0 {
            return WS2PSignal::NoConnection;
        }
        let ws2p_full_id = message.0;
        match message.1 {
            WS2PConnectionMessagePayload::WrongUrl
            | WS2PConnectionMessagePayload::FailOpenWS
            | WS2PConnectionMessagePayload::FailToSplitWS => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                    WS2PConnectionState::WSError;
                return WS2PSignal::WSError(ws2p_full_id);
            }
            WS2PConnectionMessagePayload::TryToSendConnectMess => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                    WS2PConnectionState::TryToSendConnectMess;
            }
            WS2PConnectionMessagePayload::FailSendConnectMess => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                    WS2PConnectionState::Unreachable;
            }
            WS2PConnectionMessagePayload::WebsocketOk(sender) => {
                self.websockets.insert(ws2p_full_id, sender);
            }
            WS2PConnectionMessagePayload::ValidConnectMessage(response, new_con_state) => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 = new_con_state;
                if let WS2PConnectionState::ConnectMessOk = self.ws2p_endpoints[&ws2p_full_id].1 {
                    trace!("Send: {:#?}", response);
                    self.websockets
                        .get_mut(&ws2p_full_id)
                        .expect(&format!(
                            "Fatal error : no websocket for {} !",
                            ws2p_full_id
                        ))
                        .0
                        .send_message(&Message::text(response))
                        .unwrap();
                }
            }
            WS2PConnectionMessagePayload::ValidAckMessage(r, new_con_state) => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 = new_con_state;
                if let WS2PConnectionState::AckMessOk = self.ws2p_endpoints[&ws2p_full_id].1 {
                    trace!("DEBUG : Send: {:#?}", r);
                    self.websockets
                        .get_mut(&ws2p_full_id)
                        .expect(&format!(
                            "Fatal error : no websocket for {} !",
                            ws2p_full_id
                        ))
                        .0
                        .send_message(&Message::text(r))
                        .unwrap();
                }
            }
            WS2PConnectionMessagePayload::ValidOk(new_con_state) => {
                self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 = new_con_state;
                match self.ws2p_endpoints[&ws2p_full_id].1 {
                    WS2PConnectionState::OkMessOkWaitingAckMess => {}
                    WS2PConnectionState::Established => {
                        return WS2PSignal::ConnectionEstablished(ws2p_full_id)
                    }
                    _ => {
                        self.threads_senders_channels[&ws2p_full_id]
                            .send(WS2POrderForListeningThread::Close)
                            .unwrap();
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
                    if let Some(head) = NetworkHead::from_json_value(&head) {
                        if head.verify() {
                            if head.apply(&mut self.heads_cache) {
                                applied_heads.push(head);
                            }
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
                        self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                            WS2PConnectionState::Denial
                    }
                    WS2PConnectionState::WaitingConnectMess => {
                        self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                            WS2PConnectionState::NoResponse
                    }
                    _ => {
                        self.ws2p_endpoints.get_mut(&ws2p_full_id).unwrap().1 =
                            WS2PConnectionState::Unreachable
                    }
                }
                self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::NegociationTimeout);
                return WS2PSignal::NegociationTimeout(ws2p_full_id);
            }
            WS2PConnectionMessagePayload::Timeout => {
                self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::Timeout);
                return WS2PSignal::Timeout(ws2p_full_id);
            }
            WS2PConnectionMessagePayload::UnknowMessage => warn!(
                "WS2P : Receive Unknow Message from {}.",
                &self.connections_meta_datas[&ws2p_full_id]
                    .remote_pubkey
                    .unwrap()
            ),
            WS2PConnectionMessagePayload::WrongFormatMessage => warn!(
                "WS2P : Receive Wrong Format Message from {}.",
                &self.connections_meta_datas[&ws2p_full_id]
                    .remote_pubkey
                    .unwrap()
            ),
            WS2PConnectionMessagePayload::InvalidMessage => return WS2PSignal::Empty,
            WS2PConnectionMessagePayload::Close => {
                self.close_connection(&ws2p_full_id, WS2PCloseConnectionReason::AuthMessInvalidSig)
            }
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

    pub fn send_request_to_all_connections(
        &mut self,
        ws2p_request: &NetworkRequest,
    ) -> Result<(), SendRequestError> {
        let mut count_successful_sending: usize = 0;
        let mut errors: Vec<websocket::WebSocketError> = Vec::new();
        match ws2p_request.clone() {
            NetworkRequest::GetCurrent(req_full_id, _receiver) => {
                for (ws2p_full_id, (_ep, state)) in self.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        let ws2p_request = NetworkRequest::GetCurrent(
                            ModuleReqFullId(
                                req_full_id.clone().0,
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
                                req_full_id.clone().0,
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
    }

    pub fn send_request_to_specific_node(
        &mut self,
        receiver_ws2p_full_id: &NodeFullId,
        ws2p_request: &NetworkRequest,
    ) -> Result<(), websocket::WebSocketError> {
        self.websockets
            .get_mut(receiver_ws2p_full_id)
            .unwrap()
            .0
            .send_message(&Message::text(
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

    fn connect_to_without_checking_quotas(&mut self, endpoint: NetworkEndpoint) -> () {
        // update connection state
        self.ws2p_endpoints
            .get_mut(&endpoint.node_full_id().unwrap())
            .expect("Fatal error: try to connect to unlisted endpoint ! ")
            .1 = WS2PConnectionState::TryToOpenWS;

        // get endpoint url
        let ws_url = endpoint.get_url();

        // Create WS2PConnection
        let mut conn_meta_datas = WS2PConnectionMetaData::new(
            "b60a14fd-0826-4ae0-83eb-1a92cd59fd5308535fd3-78f2-4678-9315-cd6e3b7871b1".to_string(),
        );
        conn_meta_datas.remote_pubkey = Some(endpoint.pubkey());
        conn_meta_datas.remote_uuid = Some(endpoint.node_uuid().unwrap());

        // Prepare datas for listening thread
        let mut datas_for_listening_thread = WS2PDatasForListeningThread {
            conn_meta_datas: conn_meta_datas.clone(),
            currency: self.currency.clone().unwrap(),
            key_pair: self.key_pair.unwrap(),
        };

        // Create CONNECT Message
        let mut connect_message = WS2PConnectMessageV1 {
            currency: self.currency.clone().unwrap(),
            pubkey: self.key_pair.unwrap().pubkey,
            challenge: conn_meta_datas.challenge.clone(),
            signature: None,
        };
        connect_message.signature = Some(connect_message.sign(self.key_pair.unwrap()));
        let json_connect_message =
            serde_json::to_string(&connect_message).expect("Fail to serialize CONNECT message !");

        // Log
        trace!("Try connection to {} ...", ws_url);

        // Listen incoming messages into a thread
        let sender_to_main_thread: mpsc::Sender<WS2PThreadSignal> =
            mpsc::Sender::clone(&self.main_thread_channel.0);
        let (tx2, rx2) = mpsc::channel();
        self.connections_meta_datas
            .insert(conn_meta_datas.node_full_id(), conn_meta_datas.clone());
        self.threads_senders_channels
            .insert(conn_meta_datas.node_full_id(), tx2);
        thread::spawn(move || {
            // Open websocket
            let open_ws_time = SystemTime::now();
            let client = match ClientBuilder::new(&ws_url) {
                Ok(mut client_builder) => match client_builder.connect_insecure() {
                    Ok(c) => c,
                    Err(_) => {
                        debug!("WS2PConnectResult::FailOpenWS");
                        sender_to_main_thread
                            .send(WS2PThreadSignal::WS2PConnectionMessage(
                                WS2PConnectionMessage(
                                    datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                    WS2PConnectionMessagePayload::FailOpenWS,
                                ),
                            ))
                            .unwrap_or(());
                        return ();
                    }
                },
                Err(_) => {
                    warn!("WS2PConnectResult::WrongUrl : {}", ws_url);
                    sender_to_main_thread
                        .send(WS2PThreadSignal::WS2PConnectionMessage(
                            WS2PConnectionMessage(
                                datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                WS2PConnectionMessagePayload::WrongUrl,
                            ),
                        ))
                        .unwrap_or(());

                    return ();
                }
            };
            let (mut receiver, mut sender) = match client.split() {
                Ok((mut r, mut s)) => (r, s),
                Err(_) => {
                    sender_to_main_thread
                        .send(WS2PThreadSignal::WS2PConnectionMessage(
                            WS2PConnectionMessage(
                                datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                WS2PConnectionMessagePayload::FailToSplitWS,
                            ),
                        ))
                        .unwrap_or(());
                    return ();
                }
            };

            // Send CONNECT Message
            sender_to_main_thread
                .send(WS2PThreadSignal::WS2PConnectionMessage(
                    WS2PConnectionMessage(
                        datas_for_listening_thread.conn_meta_datas.node_full_id(),
                        WS2PConnectionMessagePayload::TryToSendConnectMess,
                    ),
                ))
                .unwrap_or(());
            match sender.send_message(&Message::text(json_connect_message)) {
                Ok(_) => {
                    sender_to_main_thread
                        .send(WS2PThreadSignal::WS2PConnectionMessage(
                            WS2PConnectionMessage(
                                datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                WS2PConnectionMessagePayload::WebsocketOk(WebsocketSender(sender)),
                            ),
                        ))
                        .unwrap_or(());
                }
                Err(_) => {
                    receiver.shutdown_all().unwrap_or(());
                    sender_to_main_thread
                        .send(WS2PThreadSignal::WS2PConnectionMessage(
                            WS2PConnectionMessage(
                                datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                WS2PConnectionMessagePayload::FailSendConnectMess,
                            ),
                        ))
                        .unwrap_or(());
                    return ();
                }
            }

            let mut last_mess_time = SystemTime::now();
            let mut spam_interval = false;
            let mut spam_counter = 0;
            for incoming_message in receiver.incoming_messages() {
                // Spam ?
                if SystemTime::now().duration_since(last_mess_time).unwrap()
                    > Duration::new(*WS2P_SPAM_INTERVAL_IN_MILLI_SECS, 0)
                {
                    if spam_interval {
                        spam_counter += 1;
                    } else {
                        spam_interval = true;
                        spam_counter = 2;
                    }
                } else {
                    spam_interval = false;
                    spam_counter = 0;
                }
                // Spam ?
                if spam_counter >= *WS2P_SPAM_LIMIT {
                    thread::sleep(Duration::from_millis(*WS2P_SPAM_SLEEP_TIME_IN_SEC));
                    last_mess_time = SystemTime::now();
                } else {
                    // Negociation timeout ?
                    if datas_for_listening_thread.conn_meta_datas.state
                        != WS2PConnectionState::Established
                        && SystemTime::now().duration_since(open_ws_time).unwrap()
                            > Duration::new(*WS2P_NEGOTIATION_TIMEOUT, 0)
                    {
                        sender_to_main_thread
                            .send(WS2PThreadSignal::WS2PConnectionMessage(
                                WS2PConnectionMessage(
                                    datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                    WS2PConnectionMessagePayload::NegociationTimeout,
                                ),
                            ))
                            .unwrap_or(());
                        break;
                    }
                    // Connection timeout ?
                    else if SystemTime::now().duration_since(last_mess_time).unwrap()
                        > Duration::new(*WS2P_CONNECTION_TIMEOUT, 0)
                    {
                        sender_to_main_thread
                            .send(WS2PThreadSignal::WS2PConnectionMessage(
                                WS2PConnectionMessage(
                                    datas_for_listening_thread.conn_meta_datas.node_full_id(),
                                    WS2PConnectionMessagePayload::Timeout,
                                ),
                            ))
                            .unwrap_or(());
                        break;
                    }
                    last_mess_time = SystemTime::now();
                    match rx2.recv_timeout(Duration::from_millis(40)) {
                        Ok(s) => match s {
                            WS2POrderForListeningThread::Close => break,
                        },
                        Err(e) => {
                            match e {
                                mpsc::RecvTimeoutError::Timeout => {
                                    match incoming_message {
                                        Ok(message) => {
                                            if message.is_close() {
                                                if sender_to_main_thread
                                                    .send(WS2PThreadSignal::WS2PConnectionMessage(
                                                        WS2PConnectionMessage(
                                                            datas_for_listening_thread
                                                                .conn_meta_datas
                                                                .node_full_id(),
                                                            WS2PConnectionMessagePayload::Close,
                                                        ),
                                                    ))
                                                    .is_ok()
                                                {
                                                    break;
                                                }
                                            } else if message.is_data() {
                                                // Parse message
                                                let m = Message::from(message);
                                                let s: String = from_utf8(&m.payload)
                                                    .unwrap()
                                                    .to_string();
                                                let message: serde_json::Value =
                                                    serde_json::from_str(&s)
                                                    .unwrap();
                                                let result = sender_to_main_thread.send(
                                                    WS2PThreadSignal::WS2PConnectionMessage(
                                                        WS2PConnectionMessage(
                                                            datas_for_listening_thread
                                                                .conn_meta_datas
                                                                .node_full_id(),
                                                            datas_for_listening_thread
                                                                .conn_meta_datas
                                                                .parse_and_check_incoming_message(
                                                                    &datas_for_listening_thread
                                                                        .currency,
                                                                    datas_for_listening_thread
                                                                        .key_pair,
                                                                    &message,
                                                                ),
                                                        ),
                                                    ),
                                                );
                                                if result.is_err() {
                                                    debug!("Close ws2p connection because ws2p main thread is unrechable !");
                                                    break;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("WebSocketError : {} ! Close ", e);
                                            //receiver.shutdown_all().unwrap_or(());
                                            break;
                                        }
                                    };
                                }
                                mpsc::RecvTimeoutError::Disconnected => {
                                    break;
                                }
                            }
                        }
                    };
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    extern crate duniter_conf;
    extern crate duniter_crypto;
    extern crate duniter_dal;
    extern crate duniter_documents;
    extern crate duniter_message;
    extern crate duniter_module;
    extern crate duniter_network;

    use self::duniter_crypto::keys::ed25519;
    use self::duniter_crypto::keys::PublicKey;
    use self::duniter_dal::parsers::blocks::parse_json_block;
    use self::duniter_documents::blockchain::v10::documents::BlockDocument;
    use self::duniter_module::DuniterModule;
    use self::duniter_network::network_endpoint::{NetworkEndpoint, NetworkEndpointApi};
    use self::duniter_network::NetworkBlock;
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_parse_json_block() {
        let json_block = json!({
            "fork": false,
            "version": 10,
            "nonce": 10500000059239 as u64,
            "number": 109966,
            "powMin": 88,
            "time": 1523300656,
            "medianTime": 1523295259,
            "membersCount": 933,
            "monetaryMass": 146881563,
            "unitbase": 0,
            "issuersCount": 44,
            "issuersFrame": 221,
            "issuersFrameVar": 0,
            "currency": "g1",
            "issuer": "GRBPV3Y7PQnB9LaZhSGuS3BqBJbSHyibzYq65kTh1nQ4",
            "signature": "GCg2Lti3TdxWlhA8JF8pRI+dRQ0XZVtcC4BqO/COTpjTQFdWG6qmUNVvdeYCtR/lu1JQe3N/IhrbyV6L/6I+Cg==",
            "hash": "000000EF5B2AA849F4C3AF3D35E1284EA1F34A9F617EA806CE8371619023DC74",
            "parameters": "",
            "previousHash": "000004C00602F8A27AE078DE6351C0DDA1EA0974A78D2BEFA7DFBE7B7C3146FD",
            "previousIssuer": "5SwfQubSat5SunNafCsunEGTY93nVM4kLSsuprNqQb6S",
            "inner_hash": "61F02B1A6AE2E4B9A1FD66CE673258B4B21C0076795571EE3C9DC440DD06C46C",
            "dividend": null,
            "identities": [],
            "joiners": [],
            "actives": [],
            "leavers": [],
            "revoked": [],
            "excluded": [],
            "certifications": [
                "Hm5qjaNuHogNRdGZ4vgnLA9DMZVUu5YWzVup5mubuxCc:8AmdBsimcLziXaCS4AcVUfPx7rkjeic7482dLbBkuZw6:109964:yHKBGMeuxyIqFb295gVNK6neRC+U0tmsX1Zed3TLjS3ZZHYYycE1piLcYsTKll4ifNVp6rm+hd/CLdHYB+29CA==",
                "BncjgJeFpGsMCCsUfzNLEexjsbuX3V2mg9P67ov2LkwK:DyBUBNpzpfvjtwYYSaVMM6ST6t2DNg3NCE9CU9bRQFhF:105864:cJEGW9WxJwlMA2+4LNAK4YieyseUy1WIkFh1YLYD+JJtJEoCSnIQRXzhiAoRpGaj0bRz8sTpwI6PRkuVoDJJDQ=="
            ],
            "transactions": [
                {
                "version": 10,
                "currency": "g1",
                "locktime": 0,
                "hash": "80FE1E83DC4D0B722CA5F8363EFC6A3E29071032EBB71C1E0DF8D4FEA589C698",
                "blockstamp": "109964-00000168105D4A8A8BC8C0DC70033F45ABE472782C75A7F2074D0F4D4A3B7B2B",
                "blockstampTime": 0,
                "issuers": [
                    "6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT"
                ],
                "inputs": [
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:98284",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:98519",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:98779",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:99054",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:99326",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:99599",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:99884",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:100174",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:100469",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:100746",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:101036",
                    "1001:0:D:6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT:101327"
                ],
                "outputs": [
                    "12000:0:SIG(HmH5beJqKGMeotcQUrSW7Wo5tKvAksHmfYXfiSQ9EbWz)",
                    "12:0:SIG(6PiqcuUWhyiBF3Lgcht8c1yfk6gMfQzcUc46CqrJfeLT)"
                ],
                "unlocks": [
                    "0:SIG(0)",
                    "1:SIG(0)",
                    "2:SIG(0)",
                    "3:SIG(0)",
                    "4:SIG(0)",
                    "5:SIG(0)",
                    "6:SIG(0)",
                    "7:SIG(0)",
                    "8:SIG(0)",
                    "9:SIG(0)",
                    "10:SIG(0)",
                    "11:SIG(0)"
                ],
                "signatures": [
                    "MZxoKxYgwufh/s5mwLCsYEZXtIsP1hEKCyAzLipJsvCbR9xj7wXUw0C/ahwvZfBtR7+QVPIfLmwYEol1JcHjDw=="
                ],
                "comment": "Adhesion 2018"
                },
                {
                "version": 10,
                "currency": "g1",
                "locktime": 0,
                "hash": "B80507412B35BD5EB437AE0D3EB97E60E3A4974F5CDEA1AF7E2127C0E943481F",
                "blockstamp": "109964-00000168105D4A8A8BC8C0DC70033F45ABE472782C75A7F2074D0F4D4A3B7B2B",
                "blockstampTime": 0,
                "issuers": [
                    "8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU"
                ],
                "inputs": [
                    "1001:0:D:8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU:91560",
                    "1001:0:D:8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU:91850",
                    "1001:0:D:8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU:92111",
                    "1001:0:D:8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU:92385",
                    "1001:0:D:8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU:92635"
                ],
                "outputs": [
                    "5000:0:SIG(BzHnbec1Gov7dLSt1EzJS7vikoQCECeuvZs4wamZAcT1)",
                    "5:0:SIG(8gundJEbfm73Kx3jjw8YivJyz8qD2igjf6baCBLFCxPU)"
                ],
                "unlocks": [
                    "0:SIG(0)",
                    "1:SIG(0)",
                    "2:SIG(0)",
                    "3:SIG(0)",
                    "4:SIG(0)"
                ],
                "signatures": [
                    "A+ukwRvLWs1gZQ0KAqAnknEgmRQHdrnOvNuBx/WZqje17BAPrVxSxKpqwU6MiajU+ppigsYp6Bu0FdPf/tGnCQ=="
                ],
                "comment": ""
                },
                {
                "version": 10,
                "currency": "g1",
                "locktime": 0,
                "hash": "D8970E6629C0381A78534EEDD86803E9215A7EC4C494BAEA79EB19425F9B4D31",
                "blockstamp": "109964-00000168105D4A8A8BC8C0DC70033F45ABE472782C75A7F2074D0F4D4A3B7B2B",
                "blockstampTime": 0,
                "issuers": [
                    "FnSXE7QyBfs4ozoYAt5NEewWhHEPorf38cNXu3kX9xsg"
                ],
                "inputs": [
                    "1000:0:D:FnSXE7QyBfs4ozoYAt5NEewWhHEPorf38cNXu3kX9xsg:36597",
                    "1000:0:D:FnSXE7QyBfs4ozoYAt5NEewWhHEPorf38cNXu3kX9xsg:36880",
                    "1000:0:D:FnSXE7QyBfs4ozoYAt5NEewWhHEPorf38cNXu3kX9xsg:37082"
                ],
                "outputs": [
                    "3000:0:SIG(BBC8Rnh4CWN1wBrPLevK7GRFFVDVw7Lu24YNMUmhqoHU)"
                ],
                "unlocks": [
                    "0:SIG(0)",
                    "1:SIG(0)",
                    "2:SIG(0)"
                ],
                "signatures": [
                    "OpiF/oQfIigOeAtsteukU0w9FPSELE+BVTxhmsQ8bEeYGlwovG2VF8ZFiJkLLPi6vFuKgwzULJfjNGd97twZCw=="
                ],
                "comment": "1 billet pour une seance.pour un chouette film"
                }
            ],
        });
        let mut block: BlockDocument =
            match parse_json_block(&json_block).expect("Fail to parse test json block !") {
                NetworkBlock::V10(network_block_v10) => network_block_v10.uncompleted_block_doc,
                _ => {
                    panic!("Test block must be a v10 block !");
                }
            };
        assert_eq!(
            block.inner_hash.unwrap().to_hex(),
            "61F02B1A6AE2E4B9A1FD66CE673258B4B21C0076795571EE3C9DC440DD06C46C"
        );
        block.compute_hash();
        assert_eq!(
            block.hash.unwrap().0.to_hex(),
            "000000EF5B2AA849F4C3AF3D35E1284EA1F34A9F617EA806CE8371619023DC74"
        );
    }

    #[test]
    fn endpoint_db_tests() {
        let test_db_path = PathBuf::from("test.db");
        if test_db_path.as_path().exists() {
            fs::remove_file(&test_db_path).unwrap();
        }
        let db = WS2PModuleDatas::open_db(test_db_path).unwrap();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let mut endpoint = NetworkEndpoint::parse_from_raw(
            "WS2P cb06a19b g1.imirhil.fr 53012 /",
            ed25519::PublicKey::from_base58("5gJYnQp8v7bWwk7EWRoL8vCLof1r3y9c6VDdnGSM1GLv")
                .unwrap(),
            1,
            current_time.as_secs(),
        ).expect("Failt to parse test endpoint !");

        ws2p_db::write_endpoint(&db, &endpoint, 1, current_time.as_secs());
        let mut written_endpoints =
            ws2p_db::get_endpoints_for_api(&db, NetworkEndpointApi(String::from("WS2P")));
        assert_eq!(endpoint, written_endpoints.pop().unwrap());

        // Test status update
        endpoint.set_status(3);
        ws2p_db::write_endpoint(&db, &endpoint, 3, current_time.as_secs());
        let mut written_endpoints =
            ws2p_db::get_endpoints_for_api(&db, NetworkEndpointApi(String::from("WS2P")));
        assert_eq!(endpoint, written_endpoints.pop().unwrap());
    }

    #[test]
    fn ws2p_requests() {
        let module_id = WS2PModule::id();
        let request = NetworkRequest::GetBlocks(
            ModuleReqFullId(module_id, ModuleReqId(58)),
            NodeFullId::default(),
            50,
            0,
        );
        assert_eq!(
            network_request_to_json(&request),
            json!({
            "reqId": format!("{:x}", 58),
            "body": {
                "name": "BLOCKS_CHUNK",
                "params": {
                    "count": 50,
                    "fromNumber": 0
                }
            }
        })
        );
        assert_eq!(
            network_request_to_json(&request).to_string(),
            "{\"body\":{\"name\":\"BLOCKS_CHUNK\",\"params\":{\"count\":50,\"fromNumber\":0}},\"reqId\":\"3a\"}"
        );
    }

    #[test]
    fn ws2p_parse_head() {
        let head = json!({
            "message": "WS2POTMIC:HEAD:1:D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx:104512-0000051B9CE9C1CA89F269375A6751FB88B9E88DE47A36506057E5BFBCFBB276:c1c39a0a:duniter:1.6.21:3",
            "sig": "trtK9GXvTdfND995ohWEderpO3NkIqi1X6mBeVvMcaHckq+lIGqjWvJ9t9Vccz5t+VGaSmGUihDl4q6eldIYBw==",
            "messageV2": "WS2POTMIC:HEAD:2:D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx:104512-0000051B9CE9C1CA89F269375A6751FB88B9E88DE47A36506057E5BFBCFBB276:c1c39a0a:duniter:1.6.21:3:25:22",
            "sigV2": "x6ehPMuYjGY+z7wEGnJGyMBxMKUdu01RWaF0b0XCtoVjg67cCvT4H0V/Qcxn4bAGqzy5ux2fA7NiI+81bBnqDw==",
            "step": 0
        });
        let mut heads_count = 0;
        if let Some(head) = NetworkHead::from_json_value(&head) {
            if let NetworkHead::V2(ref head_v2) = head {
                heads_count += 1;
                assert_eq!(
                    head_v2.message.to_string(),
                    String::from("WS2POTMIC:HEAD:1:D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx:104512-0000051B9CE9C1CA89F269375A6751FB88B9E88DE47A36506057E5BFBCFBB276:c1c39a0a:duniter:1.6.21:3")
                );
            }
            assert_eq!(head.verify(), true);
        }
        assert_eq!(heads_count, 1);
    }
}

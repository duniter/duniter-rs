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

//! WebSocketToPeer API for the Duniter project.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate structopt;

extern crate byteorder;
extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate rand;
extern crate sqlite;
extern crate ws;

mod ack_message;
mod connect_message;
pub mod constants;
mod datas;
mod heads;
mod ok_message;
pub mod parsers;
pub mod serializer;
pub mod ws2p_connection;
pub mod ws2p_db;
pub mod ws2p_requests;

use ack_message::WS2PAckMessageV1;
use connect_message::WS2PConnectMessageV1;
use constants::*;
use datas::*;
use duniter_conf::DuRsConf;
use duniter_crypto::keys::*;
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALReqBlockchain, DALRequest, DALResBlockchain, DALResponse};
use duniter_documents::Blockstamp;
use duniter_message::*;
use duniter_module::*;
use duniter_network::network_endpoint::*;
use duniter_network::network_head::*;
use duniter_network::*;
use ok_message::WS2POkMessageV1;
use parsers::blocks::parse_json_block;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ws::Message;
use ws2p_connection::*;
use ws2p_requests::network_request_to_json;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// WS2P Configuration
pub struct WS2PConf {
    /// Limit of outcoming connections
    pub outcoming_quota: usize,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Vec<EndpointEnum>,
}

impl Default for WS2PConf {
    fn default() -> Self {
        WS2PConf {
            outcoming_quota: *WS2P_DEFAULT_OUTCOMING_QUOTA,
            sync_endpoints: vec![
                EndpointEnum::parse_from_raw(
                    "WS2P c1c39a0a g1-monit.librelois.fr 443 /ws2p",
                    PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(
                            "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx",
                        )
                        .unwrap(),
                    ),
                    0,
                    0,
                    1u16,
                )
                .unwrap(),
                EndpointEnum::parse_from_raw(
                    "WS2P b48824f0 g1.monnaielibreoccitanie.org 443 /ws2p",
                    PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(
                            "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef",
                        )
                        .unwrap(),
                    ),
                    0,
                    0,
                    1u16,
                )
                .unwrap(),
            ],
        }
    }
}

#[derive(Debug)]
/// Store a Signal receive from network (after message treatment)
pub enum WS2PSignal {
    /// Receive a websocket error from a connextion. `NodeFullId` store the identifier of connection.
    WSError(NodeFullId),
    /// A new connection is successfully established with `NodeFullId`.
    ConnectionEstablished(NodeFullId),
    NegociationTimeout(NodeFullId),
    Timeout(NodeFullId),
    DalRequest(NodeFullId, ModuleReqId, serde_json::Value),
    PeerCard(NodeFullId, serde_json::Value, Vec<EndpointEnum>),
    Heads(NodeFullId, Vec<NetworkHead>),
    Document(NodeFullId, NetworkDocument),
    ReqResponse(
        ModuleReqId,
        OldNetworkRequest,
        NodeFullId,
        serde_json::Value,
    ),
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
    WSError(usize, Vec<ws::Error>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WS2PModule {}

#[derive(Debug)]
pub enum WS2PThreadSignal {
    DursMsg(Box<DursMsg>),
    WS2PConnectionMessage(WS2PConnectionMessage),
}

pub trait WS2PMessage: Sized {
    fn parse(v: &serde_json::Value, currency: String) -> Option<Self>;
    fn to_raw(&self) -> String;
    fn sign(&self, key_pair: KeyPairEnum) -> Sig {
        key_pair.sign(self.to_raw().as_bytes())
    }
    fn verify(&self) -> bool;
    //fn parse_and_verify(v: serde_json::Value, currency: String) -> bool;
}

impl Default for WS2PModule {
    fn default() -> WS2PModule {
        WS2PModule {}
    }
}

#[derive(Debug)]
/// WS2PFeaturesParseError
pub enum WS2PFeaturesParseError {
    /// UnknowApiFeature
    UnknowApiFeature(String),
}

impl ApiModule<DuRsConf, DursMsg> for WS2PModule {
    type ParseErr = WS2PFeaturesParseError;
    /// Parse raw api features
    fn parse_raw_api_features(str_features: &str) -> Result<ApiFeatures, Self::ParseErr> {
        let str_features: Vec<&str> = str_features.split(' ').collect();
        let mut api_features = Vec::with_capacity(0);
        for str_feature in str_features {
            match str_feature {
                "DEF" => api_features[0] += 1u8,
                "LOW" => api_features[0] += 2u8,
                "ABF" => api_features[0] += 4u8,
                _ => {
                    return Err(WS2PFeaturesParseError::UnknowApiFeature(String::from(
                        str_feature,
                    )))
                }
            }
        }
        Ok(ApiFeatures(api_features))
    }
}

impl NetworkModule<DuRsConf, DursMsg> for WS2PModule {
    fn sync(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: WS2PConf,
        _main_sender: mpsc::Sender<RooterThreadMessage<DursMsg>>,
        _sync_params: SyncParams,
    ) -> Result<(), ModuleInitError> {
        println!("Downlaod blockchain from network...");
        println!("Error : not yet implemented !");
        Ok(())
    }
}

#[derive(StructOpt, Debug, Copy, Clone)]
#[structopt(
    name = "ws2p",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// WS2Pv1 subcommand options
pub struct WS2POpt {}

impl DuniterModule<DuRsConf, DursMsg> for WS2PModule {
    type ModuleConf = WS2PConf;
    type ModuleOpt = WS2POpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName("ws2p")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Essential()
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::NetworkKeyPair()
    }
    fn have_subcommand() -> bool {
        true
    }
    fn exec_subcommand(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        _subcommand_args: WS2POpt,
    ) {
        println!("Succesfully exec ws2p subcommand !")
    }
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        keys: RequiredKeysContent,
        conf: WS2PConf,
        rooter_sender: mpsc::Sender<RooterThreadMessage<DursMsg>>,
        load_conf_only: bool,
    ) -> Result<(), ModuleInitError> {
        // Get start time
        let start_time = SystemTime::now();

        // Define WS2PModuleDatas
        let mut ws2p_module = WS2PModuleDatas {
            rooter_sender: rooter_sender.clone(),
            key_pair: None,
            currency: None,
            conf,
            node_id: NodeId(soft_meta_datas.conf.my_node_id()),
            main_thread_channel: mpsc::channel(),
            ws2p_endpoints: HashMap::new(),
            websockets: HashMap::new(),
            requests_awaiting_response: HashMap::new(),
            heads_cache: HashMap::new(),
            my_head: None,
            uids_cache: HashMap::new(),
            count_dal_requests: 0,
        };

        // load conf
        let key_pair = match keys {
            RequiredKeysContent::NetworkKeyPair(key_pair) => key_pair,
            _ => panic!("WS2PModule fatal error at load_conf() : keys != NetworkKeyPair"),
        };
        let mut ws2p_endpoints = HashMap::new();
        for ep in ws2p_module.conf.sync_endpoints.clone() {
            ws2p_endpoints.insert(
                ep.node_full_id()
                    .expect("Fail to get endpoint node_full_id"),
                (ep.clone(), WS2PConnectionState::Close),
            );
            info!("Load sync endpoint {}", ep.raw());
        }
        ws2p_module.key_pair = Some(key_pair);
        ws2p_module.currency = Some(soft_meta_datas.conf.currency().to_string());
        ws2p_module.ws2p_endpoints = ws2p_endpoints;

        // Create ws2p main thread channel
        let ws2p_sender_clone = ws2p_module.main_thread_channel.0.clone();

        // Create proxy channel
        let (proxy_sender, proxy_receiver): (
            mpsc::Sender<DursMsg>,
            mpsc::Receiver<DursMsg>,
        ) = mpsc::channel();
        let proxy_sender_clone = proxy_sender.clone();

        // Launch a proxy thread that transform DursMsg to WS2PThreadSignal(DursMsg)
        thread::spawn(move || {
            // Send proxy sender to main
            rooter_sender
                .send(RooterThreadMessage::ModuleSender(
                    WS2PModule::name(),
                    proxy_sender_clone,
                    vec![ModuleRole::InterNodesNetwork],
                    vec![
                        ModuleEvent::NewValidBlock,
                        ModuleEvent::NewWotDocInPool,
                        ModuleEvent::NewTxinPool,
                    ],
                ))
                .expect("Fatal error : ws2p module fail to send is sender channel !");
            debug!("Send ws2p sender to main thread.");
            loop {
                match proxy_receiver.recv() {
                    Ok(message) => {
                        let stop = if let DursMsgContent::Stop() = message.1 {
                            true
                        } else {
                            false
                        };
                        ws2p_sender_clone
                            .send(WS2PThreadSignal::DursMsg(Box::new(message)))
                            .expect(
                                "Fatal error : fail to relay DursMsgContent to ws2p main thread !",
                            );
                        if stop {
                            break;
                        };
                    }
                    Err(e) => panic!(format!("{}", e)),
                }
            }
        });

        // open ws2p bdd
        let mut db_path =
            duniter_conf::datas_path(&soft_meta_datas.profile, &soft_meta_datas.conf.currency());
        db_path.push("ws2p.db");
        let db = WS2PModuleDatas::open_db(&db_path).expect("Fatal error : fail to open WS2P DB !");

        // Get ws2p endpoints in BDD
        let mut count = 0;
        let dal_enpoints =
            ws2p_db::get_endpoints_for_api(&db, &NetworkEndpointApi(String::from("WS2P")));
        for ep in dal_enpoints {
            if ep.api() == NetworkEndpointApi(String::from("WS2P"))
                && (cfg!(feature = "ssl") || ep.port() != 443)
            {
                count += 1;
                ws2p_module.ws2p_endpoints.insert(
                    ep.node_full_id()
                        .expect("WS2P: Fail to get ep.node_full_id() !"),
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
                    WS2PThreadSignal::DursMsg(ref duniter_mesage) => {
                        match (*duniter_mesage.deref()).1 {
                            DursMsgContent::Stop() => break,
                            DursMsgContent::OldNetworkRequest(ref request) => match *request {
                                OldNetworkRequest::GetBlocks(
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
                                                    &OldNetworkRequest::GetBlocks(
                                                        *req_id, *receiver, *count, *from,
                                                    ),
                                                );
                                        }
                                    } else {
                                        let _blocks_request_result = ws2p_module
                                            .send_request_to_specific_node(
                                                &receiver,
                                                &OldNetworkRequest::GetBlocks(
                                                    *req_id, *receiver, *count, *from,
                                                ),
                                            );
                                    }
                                }
                                OldNetworkRequest::GetEndpoints(ref _request) => {}
                                _ => {}
                            },
                            DursMsgContent::DALEvent(ref dal_event) => match *dal_event {
                                DALEvent::StackUpValidBlock(ref _block, ref blockstamp) => {
                                    current_blockstamp = *blockstamp;
                                    debug!(
                                        "WS2PModule : current_blockstamp = {}",
                                        current_blockstamp
                                    );
                                    ws2p_module.my_head = Some(heads::generate_my_head(
                                        &key_pair,
                                        NodeId(soft_meta_datas.conf.my_node_id()),
                                        soft_meta_datas.soft_name,
                                        soft_meta_datas.soft_version,
                                        &current_blockstamp,
                                        None,
                                    ));
                                    ws2p_module.send_network_event(&NetworkEvent::ReceiveHeads(
                                        vec![ws2p_module.my_head.clone().unwrap()],
                                    ));
                                    // Send my head to all connections
                                    let my_json_head = serializer::serialize_head(
                                        ws2p_module.my_head.clone().unwrap(),
                                    );
                                    trace!("Send my HEAD: {:#?}", my_json_head);
                                    let _results: Result<
                                        (),
                                        ws::Error,
                                    > = ws2p_module
                                        .websockets
                                        .iter_mut()
                                        .map(|ws| {
                                            (ws.1).0.send(Message::text(
                                                json!({
                                                "name": "HEAD",
                                                "body": {
                                                    "heads": [my_json_head]
                                                }
                                            })
                                                .to_string(),
                                            ))
                                        })
                                        .collect();
                                }
                                DALEvent::RevertBlocks(ref _blocks) => {}
                                _ => {}
                            },
                            DursMsgContent::DALResponse(ref dal_res) => match *dal_res.deref() {
                                DALResponse::Blockchain(ref dal_res_bc) => {
                                    match *dal_res_bc.deref() {
                                        DALResBlockchain::CurrentBlock(
                                            ref _requester_full_id,
                                            ref current_block,
                                            ref current_blockstamp_,
                                        ) => {
                                            let _current_block = current_block.deref();
                                            debug!(
                                                "WS2PModule : receive DALResBc::CurrentBlock({})",
                                                current_blockstamp
                                            );
                                            current_blockstamp = *current_blockstamp_;
                                            if ws2p_module.my_head.is_none() {
                                                ws2p_module.my_head =
                                                    Some(heads::generate_my_head(
                                                        &key_pair,
                                                        NodeId(soft_meta_datas.conf.my_node_id()),
                                                        soft_meta_datas.soft_name,
                                                        soft_meta_datas.soft_version,
                                                        &current_blockstamp,
                                                        None,
                                                    ));
                                            }
                                            ws2p_module.send_network_event(
                                                &NetworkEvent::ReceiveHeads(vec![ws2p_module
                                                    .my_head
                                                    .clone()
                                                    .unwrap()]),
                                            );
                                        }
                                        DALResBlockchain::UIDs(ref _req_id, ref uids) => {
                                            // Add uids to heads
                                            for head in ws2p_module.heads_cache.values_mut() {
                                                if let Some(uid_option) = uids.get(&head.pubkey()) {
                                                    if let Some(ref uid) = *uid_option {
                                                        head.set_uid(uid);
                                                        ws2p_module
                                                            .uids_cache
                                                            .insert(head.pubkey(), uid.to_string());
                                                    } else {
                                                        ws2p_module
                                                            .uids_cache
                                                            .remove(&head.pubkey());
                                                    }
                                                }
                                            }
                                            // Resent heads to other modules
                                            ws2p_module.send_network_event(
                                                &NetworkEvent::ReceiveHeads(
                                                    ws2p_module
                                                        .heads_cache
                                                        .values()
                                                        .cloned()
                                                        .collect(),
                                                ),
                                            );
                                            // Resent to other modules connections that match receive uids
                                            for (node_full_id, (ep, conn_state)) in
                                                &ws2p_module.ws2p_endpoints
                                            {
                                                if let Some(uid_option) = uids.get(&node_full_id.1)
                                                {
                                                    ws2p_module.send_network_event(
                                                        &NetworkEvent::ConnectionStateChange(
                                                            *node_full_id,
                                                            *conn_state as u32,
                                                            uid_option.clone(),
                                                            ep.get_url(false, false)
                                                                .expect("Endpoint unreachable !"),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                DALResponse::Pendings(_) => {}
                            },
                            _ => {}
                        }
                    }
                    WS2PThreadSignal::WS2PConnectionMessage(ws2p_conn_message) => match ws2p_module
                        .ws2p_conn_message_pretreatment(ws2p_conn_message)
                    {
                        WS2PSignal::NoConnection => {
                            warn!("WS2PSignal::NoConnection");
                        }
                        WS2PSignal::ConnectionEstablished(ws2p_full_id) => {
                            let req_id =
                                ModuleReqId(ws2p_module.requests_awaiting_response.len() as u32);
                            let module_id = WS2PModule::name();
                            let _current_request_result = ws2p_module
                                .send_request_to_specific_node(
                                    &ws2p_full_id,
                                    &OldNetworkRequest::GetCurrent(
                                        ModuleReqFullId(module_id, req_id),
                                        ws2p_full_id,
                                    ),
                                );
                            if ws2p_module.uids_cache.get(&ws2p_full_id.1).is_none() {
                                ws2p_module.send_dal_request(&DALRequest::BlockchainRequest(
                                    DALReqBlockchain::UIDs(vec![ws2p_full_id.1]),
                                ));
                            }
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Established as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                                ws2p_module.ws2p_endpoints[&ws2p_full_id]
                                    .0
                                    .get_url(false, false)
                                    .expect("Endpoint unreachable !"),
                            ));
                        }
                        WS2PSignal::WSError(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::WSError as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                                ws2p_module.ws2p_endpoints[&ws2p_full_id]
                                    .0
                                    .get_url(false, false)
                                    .expect("Endpoint unreachable !"),
                            ));
                        }
                        WS2PSignal::NegociationTimeout(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Denial as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                                ws2p_module.ws2p_endpoints[&ws2p_full_id]
                                    .0
                                    .get_url(false, false)
                                    .expect("Endpoint unreachable !"),
                            ));
                        }
                        WS2PSignal::Timeout(ws2p_full_id) => {
                            endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                            ws2p_module.send_network_event(&NetworkEvent::ConnectionStateChange(
                                ws2p_full_id,
                                WS2PConnectionState::Close as u32,
                                ws2p_module.uids_cache.get(&ws2p_full_id.1).cloned(),
                                ws2p_module.ws2p_endpoints[&ws2p_full_id]
                                    .0
                                    .get_url(false, false)
                                    .expect("Endpoint unreachable !"),
                            ));
                        }
                        WS2PSignal::PeerCard(_ws2p_full_id, _peer_card, ws2p_endpoints) => {
                            //trace!("WS2PSignal::PeerCard({})", ws2p_full_id);
                            //ws2p_module.send_network_event(NetworkEvent::ReceivePeers(_));
                            for ep in ws2p_endpoints {
                                match ws2p_module.ws2p_endpoints.get(
                                    &ep.node_full_id()
                                        .expect("WS2P: Fail to get ep.node_full_id() !"),
                                ) {
                                    Some(_) => {}
                                    None => {
                                        if let Some(_api) =
                                            ws2p_db::string_to_api(&ep.api().0.clone())
                                        {
                                            endpoints_to_update_status.insert(
                                                ep.node_full_id().expect(
                                                    "WS2P: Fail to get ep.node_full_id() !",
                                                ),
                                                SystemTime::now(),
                                            );
                                        }
                                        if cfg!(feature = "ssl") || ep.port() != 443 {
                                            ws2p_module.connect_to(&ep);
                                        }
                                    }
                                };
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
                                OldNetworkRequest::GetCurrent(ref _req_id, _receiver) => {
                                    info!(
                                        "WS2PSignal::ReceiveCurrent({}, {:?}, {:#?})",
                                        req_id.0, req, response
                                    );
                                    if let Some(block) = parse_json_block(&response) {
                                        ws2p_module.send_network_req_response(
                                            req.get_req_full_id().0,
                                            NetworkResponse::CurrentBlock(
                                                ModuleReqFullId(WS2PModule::name(), req_id),
                                                recipient_full_id,
                                                Box::new(block),
                                            ),
                                        );
                                    }
                                }
                                OldNetworkRequest::GetBlocks(
                                    ref _req_id,
                                    _receiver,
                                    _count,
                                    from,
                                ) => {
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
                                OldNetworkRequest::GetRequirementsPending(
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
                .unwrap()
                > Duration::new(5, 0)
            {
                last_ws2p_connections_print = SystemTime::now();
                let mut connected_nodes = Vec::new();
                for (k, (_ep, state)) in ws2p_module.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        connected_nodes.push(k);
                    }
                }
                // Print network consensus
                match ws2p_module.get_network_consensus() {
                    Ok(consensus_blockstamp) => {
                        debug!(
                            "WS2PModule : get_network_consensus() = {:?}",
                            consensus_blockstamp
                        );
                        if current_blockstamp.id.0 < (consensus_blockstamp.id.0 + 2) {
                            warn!("We probably are in a fork branch !");
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
                if connected_nodes.len() < ws2p_module.conf.clone().outcoming_quota
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
                    let module_id = WS2PModule::name();
                    let _blocks_request_result =
                        ws2p_module.send_request_to_all_connections(&OldNetworkRequest::GetBlocks(
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
                        &OldNetworkRequest::GetRequirementsPending(ModuleReqId(0 as u32), 5),
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

#[cfg(test)]
mod tests {
    extern crate duniter_conf;
    extern crate duniter_crypto;
    extern crate duniter_dal;
    extern crate duniter_documents;
    extern crate duniter_message;
    extern crate duniter_module;
    extern crate duniter_network;

    use super::parsers::blocks::parse_json_block;
    use super::*;
    use duniter_crypto::keys::PublicKey;
    use duniter_documents::blockchain::v10::documents::BlockDocument;
    use duniter_module::DuniterModule;
    use duniter_network::network_endpoint::{EndpointEnum, NetworkEndpointApi};
    use duniter_network::NetworkBlock;
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
            block
                .inner_hash
                .expect("Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            "61F02B1A6AE2E4B9A1FD66CE673258B4B21C0076795571EE3C9DC440DD06C46C"
        );
        block.compute_hash();
        assert_eq!(
            block
                .hash
                .expect("Try to get hash of an uncompleted or reduce block !")
                .0
                .to_hex(),
            "000000EF5B2AA849F4C3AF3D35E1284EA1F34A9F617EA806CE8371619023DC74"
        );
    }

    #[test]
    fn endpoint_db_tests() {
        let test_db_path = PathBuf::from("test.db");
        if test_db_path.as_path().exists() {
            fs::remove_file(&test_db_path).unwrap();
        }
        let db = WS2PModuleDatas::open_db(&test_db_path).unwrap();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let mut endpoint = EndpointEnum::parse_from_raw(
            "WS2P cb06a19b g1.imirhil.fr 53012 /",
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58("5gJYnQp8v7bWwk7EWRoL8vCLof1r3y9c6VDdnGSM1GLv")
                    .unwrap(),
            ),
            1,
            current_time.as_secs(),
            1,
        )
        .expect("Failt to parse test endpoint !");

        ws2p_db::write_endpoint(&db, &endpoint, 1, current_time.as_secs());
        let mut written_endpoints =
            ws2p_db::get_endpoints_for_api(&db, &NetworkEndpointApi(String::from("WS2P")));
        assert_eq!(endpoint, written_endpoints.pop().unwrap());

        // Test status update
        endpoint.set_status(3);
        ws2p_db::write_endpoint(&db, &endpoint, 3, current_time.as_secs());
        let mut written_endpoints =
            ws2p_db::get_endpoints_for_api(&db, &NetworkEndpointApi(String::from("WS2P")));
        assert_eq!(endpoint, written_endpoints.pop().unwrap());
    }

    #[test]
    fn ws2p_requests() {
        let module_id = WS2PModule::name();
        let request = OldNetworkRequest::GetBlocks(
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
        if let Ok(head) = NetworkHead::from_json_value(&head) {
            if let NetworkHead::V2(ref head_v2) = head {
                heads_count += 1;
                assert_eq!(
                    head_v2.message.to_string(),
                    String::from("WS2POTMIC:HEAD:1:D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx:104512-0000051B9CE9C1CA89F269375A6751FB88B9E88DE47A36506057E5BFBCFBB276:c1c39a0a:duniter:1.6.21:3")
                );
            }
            assert_eq!(head.verify(), true);
        } else {
            panic!("Fail to parse head !")
        }
        assert_eq!(heads_count, 1);
    }
}

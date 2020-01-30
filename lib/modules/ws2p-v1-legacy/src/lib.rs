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

//! WebSocketToPeer API for the Dunitrust project.

#![allow(clippy::large_enum_variant, clippy::identity_conversion)]
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
extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate structopt;

mod ack_message;
mod connect_message;
pub mod constants;
mod events;
mod heads;
mod ok_message;
mod requests;
mod responses;
pub mod serializers;
mod subcommands;
pub mod ws2p_db;
pub mod ws_connections;

use crate::ack_message::WS2PAckMessageV1;
use crate::connect_message::WS2PConnectMessageV1;
use crate::constants::*;
use crate::ok_message::WS2POkMessageV1;
use crate::requests::sent::send_dal_request;
use crate::subcommands::WS2PSubCommands;
use crate::ws2p_db::DbEndpoint;
use crate::ws_connections::messages::WS2Pv1Msg;
use crate::ws_connections::requests::{WS2Pv1ReqBody, WS2Pv1ReqFullId, WS2Pv1ReqId, WS2Pv1Request};
use crate::ws_connections::states::WS2PConnectionState;
use crate::ws_connections::*;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::Blockstamp;
use dubp_currency_params::CurrencyName;
use dubp_user_docs::documents::UserDocumentDUBP;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use durs_common_tools::traits::merge::Merge;
use durs_conf::DuRsConf;
use durs_message::events::*;
use durs_message::requests::*;
use durs_message::responses::*;
use durs_message::*;
use durs_module::*;
use durs_network::cli::sync::SyncOpt;
use durs_network::events::*;
use durs_network::requests::*;
use durs_network::*;
use durs_network_documents::network_endpoint::*;
use durs_network_documents::network_head::*;
use durs_network_documents::*;
use failure::Fail;
use maplit::hashset;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use unwrap::unwrap;
use ws::{CloseCode, Message};

#[inline]
#[cfg(not(feature = "ssl"))]
pub fn ssl() -> bool {
    false
}
#[inline]
#[cfg(feature = "ssl")]
pub fn ssl() -> bool {
    true
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// WS2P Configuration
pub struct WS2PUserConf {
    /// Limit of outcoming connections
    pub outcoming_quota: Option<usize>,
    /// List of prefered public keys
    pub prefered_pubkeys: Option<HashSet<String>>,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Option<Vec<EndpointV1>>,
}

impl Merge for WS2PUserConf {
    fn merge(self, other: Self) -> Self {
        WS2PUserConf {
            outcoming_quota: self.outcoming_quota.or(other.outcoming_quota),
            prefered_pubkeys: self.prefered_pubkeys.or(other.prefered_pubkeys),
            sync_endpoints: self.sync_endpoints.or(other.sync_endpoints),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// WS2P Configuration
pub struct WS2PConf {
    /// Currency name
    pub currency: Option<CurrencyName>,
    /// Limit of outcoming connections
    pub outcoming_quota: usize,
    /// List of prefered public keys
    pub prefered_pubkeys: HashSet<PubKey>,
    /// Default WS2P endpoints provides by configuration file
    pub sync_endpoints: Vec<EndpointV1>,
}

impl Default for WS2PConf {
    fn default() -> Self {
        WS2PConf {
            currency: None,
            outcoming_quota: *WS2P_DEFAULT_OUTCOMING_QUOTA,
            prefered_pubkeys: HashSet::new(),
            sync_endpoints: vec![
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P e66254bf 91.121.157.13 20901",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "8iVdpXqFLCxGyPqgVx5YbFSkmWKkceXveRd2yvBKeARL",
                    )),),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P c1c39a0a ts.g1.librelois.fr 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx",
                    )),),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P fb17fcd4 g1.duniter.fr 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "38MEAZN68Pz1DTvT3tqgxx4yQP6snJCQhPqEFxbDk4aE",
                    ))),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P 9407e0ac monit.g1.nordstrom.duniter.org 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "74RBUM4VkhZU4PLJcf8ok9snKjXTX6aP52PdGcCM1meA",
                    ))),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P beb7012c g1.monnaielibreoccitanie.org 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "RD11hyG5HY9MGAp4ui3KoPYWHBMEBCyzSPT81Em4cCL",
                    ))),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P 90e9b12 duniter.g1.1000i100.fr 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "2sZF6j2PkxBDNAqUde7Dgo5x3crkerZpQ4rBqqJGn8QT",
                    ))),
                    0,
                    0,
                )),
                unwrap!(EndpointV1::parse_from_raw(
                    "WS2P dff60418 duniter.normandie-libre.fr 443 /ws2p",
                    PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        "8t6Di3pLxxoTEfjXHjF49pNpjSTXuGEQ6BpkT75CkNb2",
                    ))),
                    0,
                    0,
                )),
            ],
        }
    }
}

#[derive(Debug)]
/// Store a Signal receive from network (after message treatment)
pub enum WS2PSignal {
    Blocks(NodeFullId, Vec<BlockDocument>),
    /// A new connection is successfully established with `NodeFullId`.
    ConnectionEstablished(NodeFullId),
    Empty,
    Heads(NodeFullId, Vec<NetworkHead>),
    NegociationTimeout(NodeFullId),
    NoConnection,
    PeerCard(NodeFullId, serde_json::Value, Vec<EndpointV1>),
    ReqResponse(
        ModuleReqFullId,
        WS2Pv1ReqBody,
        NodeFullId,
        serde_json::Value,
    ),
    Request {
        from: NodeFullId,
        req_id: WS2Pv1ReqId,
        body: WS2Pv1ReqBody,
    },
    Timeout(NodeFullId),
    UserDocuments(NodeFullId, Vec<UserDocumentDUBP>),
    /// Receive a websocket error from a connextion. `NodeFullId` store the identifier of connection.
    WSError(NodeFullId),
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

#[derive(Debug)]
pub struct WS2Pv1Module {
    pub conf: WS2PConf,
    pub count_dal_requests: u32,
    pub current_blockstamp: Blockstamp,
    pub ep_file_path: PathBuf,
    pub heads_cache: HashMap<NodeFullId, NetworkHead>,
    pub key_pair: KeyPairEnum,
    pub main_thread_channel: (
        mpsc::Sender<WS2PThreadSignal>,
        mpsc::Receiver<WS2PThreadSignal>,
    ),
    pub my_head: Option<NetworkHead>,
    pub my_signator: SignatorEnum,
    pub next_receiver: usize,
    pub node_id: NodeId,
    pub pending_received_requests: HashMap<ModuleReqId, WS2Pv1ReqFullId>,
    pub requests_awaiting_response: HashMap<WS2Pv1ReqId, WS2Pv1PendingReqInfos>,
    pub router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    pub soft_name: &'static str,
    pub soft_version: &'static str,
    pub ssl: bool,
    pub websockets: HashMap<NodeFullId, WsSender>,
    pub ws2p_endpoints: HashMap<NodeFullId, DbEndpoint>,
    pub uids_cache: HashMap<PubKey, String>,
}

#[derive(Copy, Clone, Debug)]
pub struct WS2Pv1PendingReqInfos {
    requester_module: ModuleReqFullId,
    req_body: WS2Pv1ReqBody,
    recipient_node: NodeFullId,
    timestamp: SystemTime,
}

impl WS2Pv1Module {
    pub fn new(
        soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        conf: WS2PConf,
        ep_file_path: PathBuf,
        key_pair: KeyPairEnum,
        router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    ) -> WS2Pv1Module {
        let my_signator = if let Ok(signator) = key_pair.generate_signator() {
            signator
        } else {
            fatal_error!("Your key pair is corrupted, please recreate it !");
        };

        WS2Pv1Module {
            router_sender,
            key_pair,
            current_blockstamp: Blockstamp::default(),
            conf,
            ep_file_path,
            soft_name: soft_meta_datas.soft_name,
            soft_version: soft_meta_datas.soft_version,
            ssl: ssl(),
            node_id: NodeId(soft_meta_datas.conf.my_node_id()),
            main_thread_channel: mpsc::channel(),
            next_receiver: 0,
            pending_received_requests: HashMap::new(),
            ws2p_endpoints: HashMap::new(),
            websockets: HashMap::new(),
            requests_awaiting_response: HashMap::new(),
            heads_cache: HashMap::new(),
            my_head: None,
            my_signator,
            uids_cache: HashMap::new(),
            count_dal_requests: 0,
        }
    }
}

#[derive(Debug)]
pub enum WS2PThreadSignal {
    DursMsg(Box<DursMsg>),
    WS2Pv1Msg(WS2Pv1Msg),
}

#[derive(Copy, Clone, Debug)]
/// Error when parsing WS2P message
pub struct WS2PMsgParseErr {}

impl From<dup_crypto::bases::BaseConvertionError> for WS2PMsgParseErr {
    fn from(_: dup_crypto::bases::BaseConvertionError) -> Self {
        WS2PMsgParseErr {}
    }
}

pub trait WS2PMessage: Sized {
    fn parse(v: &serde_json::Value, currency: String) -> Result<Self, WS2PMsgParseErr>;
    fn to_raw(&self) -> String;
    fn sign(&self, signator: &SignatorEnum) -> Sig {
        signator.sign(self.to_raw().as_bytes())
    }
    fn verify(&self) -> bool;
    //fn parse_and_verify(v: serde_json::Value, currency: String) -> bool;
}

#[derive(Debug)]
/// WS2PFeaturesParseError
pub enum WS2PFeaturesParseError {
    /// UnknowApiFeature
    UnknowApiFeature(String),
}

impl ApiModule<DuRsConf, DursMsg> for WS2Pv1Module {
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
                    )));
                }
            }
        }
        Ok(ApiFeatures(api_features))
    }
}

impl NetworkModule<DuRsConf, DursMsg> for WS2Pv1Module {
    fn sync(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _conf: WS2PConf,
        _main_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
        _sync_params: SyncOpt,
    ) -> Result<(), SyncError> {
        println!("Downlaod blockchain from network...");
        println!("Error : not yet implemented !");
        Ok(())
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "ws2p", setting(structopt::clap::AppSettings::ColoredHelp))]
/// WS2P1 subcommand options
pub struct WS2POpt {
    /// Ws2p1 subcommands
    #[structopt(subcommand)]
    pub subcommand: WS2PSubCommands,
}

macro_rules! fields_overload {
    ($struct:ident; $option_struct:ident; [$($field:ident),+]) => {{
        $(if let Some($field) = $option_struct.$field {
            $struct.$field = $field;
        })+
    }};
}

#[derive(Clone, Debug, Fail)]
enum WS2Pv1Error {
    #[fail(display = "WS2Pv1Module fatal error at load_conf() : keys != NetworkKeyPair")]
    UnexpectedKeys,
}

impl DursModule<DuRsConf, DursMsg> for WS2Pv1Module {
    type ModuleUserConf = WS2PUserConf;
    type ModuleConf = WS2PConf;
    type ModuleOpt = WS2POpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName(MODULE_NAME)
    }
    fn priority() -> ModulePriority {
        ModulePriority::Essential
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::NetworkKeyPair
    }
    fn have_subcommand() -> bool {
        true
    }

    fn generate_module_conf(
        currency_name: Option<&CurrencyName>,
        _global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
        module_user_conf: Option<Self::ModuleUserConf>,
    ) -> Result<(Self::ModuleConf, Option<Self::ModuleUserConf>), ModuleConfError> {
        let mut conf = WS2PConf::default();
        conf.currency = currency_name.cloned();

        if currency_name.is_some() && unwrap!(currency_name) == &CurrencyName("g1-test".to_owned())
        {
            conf.sync_endpoints = vec![unwrap!(EndpointV1::parse_from_raw(
                "WS2P 17ae4dd9 ts.gt.elo.tf 80 ws2p",
                PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                    "42jMJtb8chXrpHMAMcreVdyPJK7LtWjEeRqkPw4eSEVp",
                )),),
                0,
                0,
            ))];
        }

        if let Some(module_user_conf) = module_user_conf.clone() {
            /*if let Some(outcoming_quota) = module_user_conf.outcoming_quota {
                conf.outcoming_quota = outcoming_quota;
            }
            if let Some(sync_endpoints) = module_user_conf.sync_endpoints {
                conf.sync_endpoints = sync_endpoints;
            }*/
            if let Some(prefered_pubkeys) = module_user_conf.prefered_pubkeys {
                conf.prefered_pubkeys = prefered_pubkeys
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        PubKey::from_str(p).map_err(|e| ModuleConfError::InvalidField {
                            field_name: stringify!(prefered_pubkeys),
                            cause: format!("pubkey n°{} is invalid: {}", i, e),
                        })
                    })
                    .collect::<Result<HashSet<PubKey>, ModuleConfError>>()?;
            }
            fields_overload!(
                conf;
                module_user_conf;
                [
                    outcoming_quota,
                    sync_endpoints
                ]
            )
        }

        Ok((conf, module_user_conf))
    }
    fn exec_subcommand(
        _soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        module_user_conf: Option<Self::ModuleUserConf>,
        opts: WS2POpt,
    ) -> Option<Self::ModuleUserConf> {
        match opts.subcommand {
            WS2PSubCommands::Prefered {
                subcommand: prefered_subcommand,
            } => prefered_subcommand.execute(module_user_conf),
        }
    }
    fn start(
        soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
        keys: RequiredKeysContent,
        conf: WS2PConf,
        router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    ) -> Result<(), failure::Error> {
        // Get start time
        let start_time = SystemTime::now();

        // Get key_pair
        let key_pair = if let RequiredKeysContent::NetworkKeyPair(key_pair) = keys {
            key_pair
        } else {
            return Err(WS2Pv1Error::UnexpectedKeys.into());
        };

        // load conf
        let mut ws2p_endpoints = HashMap::new();
        for ep in &conf.sync_endpoints {
            info!("Load sync endpoint {}", ep.raw_endpoint);
            let node_full_id = ep
                .node_full_id()
                .expect("Fail to get endpoint node_full_id");
            ws2p_endpoints.insert(
                node_full_id,
                DbEndpoint {
                    ep: ep.clone(),
                    state: WS2PConnectionState::Close,
                    last_check: 0,
                },
            );
        }

        // Get endpoints file path
        let mut ep_file_path = durs_conf::get_datas_path(soft_meta_datas.profile_path.clone());
        ep_file_path.push("ws2pv1");
        if !ep_file_path.exists() {
            fs::create_dir(ep_file_path.as_path()).expect("Impossible to create ws2pv1 dir !");
        }
        ep_file_path.push("endpoints.bin");

        // Define WS2Pv1Module
        let mut ws2p_module = WS2Pv1Module::new(
            soft_meta_datas,
            conf,
            ep_file_path.clone(),
            key_pair,
            router_sender.clone(),
        );
        ws2p_module.ws2p_endpoints = ws2p_endpoints;

        // Create ws2p main thread channel
        let ws2p_sender_clone = ws2p_module.main_thread_channel.0.clone();

        // Get ws2p endpoints in file
        debug!("WS2P SSL={}", ssl());
        let count;
        match ws2p_db::get_endpoints(&ep_file_path) {
            Ok(ws2p_enpoints) => {
                let ws2p_enpoints = ws2p_enpoints
                    .into_iter()
                    .filter(|(_, dal_ep)| cfg!(feature = "ssl") || dal_ep.ep.port != 443)
                    .map(|(node_full_id, mut dal_ep)| {
                        if dal_ep.state == WS2PConnectionState::Established {
                            dal_ep.state = WS2PConnectionState::Close;
                        }
                        (node_full_id, dal_ep)
                    })
                    .collect::<Vec<(NodeFullId, DbEndpoint)>>();
                count = ws2p_enpoints.len();
                ws2p_module.ws2p_endpoints.extend(ws2p_enpoints);
            }
            Err(err) => fatal_error!("WS2Pv1: fail to load endpoints from DB: {:?}", err),
        }
        info!("Load {} endpoints from DB !", count);

        // Create proxy channel
        let (proxy_sender, proxy_receiver): (mpsc::Sender<DursMsg>, mpsc::Receiver<DursMsg>) =
            mpsc::channel();
        let proxy_sender_clone = proxy_sender;

        // Launch a proxy thread that transform DursMsg to WS2PThreadSignal(DursMsg)
        thread::spawn(move || {
            // Send proxy sender to main
            router_sender
                .send(RouterThreadMessage::ModuleRegistration {
                    static_name: WS2Pv1Module::name(),
                    sender: proxy_sender_clone,
                    roles: vec![ModuleRole::InterNodesNetwork],
                    events_subscription: vec![
                        ModuleEvent::CurrencyParameters,
                        ModuleEvent::NewValidBlock,
                        ModuleEvent::NewWotDocInPool,
                        ModuleEvent::NewTxinPool,
                    ],
                    reserved_apis_parts: vec![ApiPart {
                        name: ApiName(WS2P_API.to_owned()),
                        versions: hashset![ApiVersion(1)],
                    }],
                    endpoints: vec![],
                })
                .expect("Fatal error : ws2p1 module fail to send is sender channel !");
            debug!("Send ws2p1 sender to main thread.");
            loop {
                match proxy_receiver.recv() {
                    Ok(message) => {
                        let stop = if let DursMsg::Stop = message {
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
                    Err(e) => fatal_error!(format!("{}", e)),
                }
            }
        });

        // Request current blockstamp
        send_dal_request(&mut ws2p_module, &BlockchainRequest::CurrentBlockstamp());

        // Start
        connect_to_know_endpoints(&mut ws2p_module);
        ws2p_module.main_loop(start_time);

        Ok(())
    }
}

impl WS2Pv1Module {
    fn main_loop(mut self, start_time: SystemTime) {
        // Initialize variables
        let mut last_ws2p_connecting_wave = SystemTime::now();
        let mut last_ws2p_state_print = SystemTime::now();
        let mut last_ws2p_endpoints_write = SystemTime::now();
        let mut endpoints_to_update_status: HashMap<NodeFullId, SystemTime> = HashMap::new();
        let mut last_identities_request = UNIX_EPOCH;

        loop {
            match self
                .main_thread_channel
                .1
                .recv_timeout(Duration::from_millis(200))
            {
                Ok(message) => match message {
                    WS2PThreadSignal::DursMsg(durs_mesage) => {
                        match durs_mesage.deref() {
                            DursMsg::Stop => {
                                // Close all connections
                                for ws in self.websockets.values() {
                                    let _ = ws.0.close(CloseCode::Normal);
                                }
                                // Break main loop
                                break;
                            }
                            DursMsg::Request {
                                ref req_content, ..
                            } => requests::received::receive_req(&mut self, req_content),
                            DursMsg::Event {
                                ref event_type,
                                ref event_content,
                                ..
                            } => events::received::receive_event(
                                &mut self,
                                *event_type,
                                event_content,
                            ),
                            DursMsg::Response {
                                req_id,
                                res_content,
                                ..
                            } => {
                                responses::received::receive_response(
                                    &mut self,
                                    *req_id,
                                    res_content,
                                );
                            }
                            _ => {} // Others DursMsg variants
                        }
                    }
                    WS2PThreadSignal::WS2Pv1Msg(msg) => {
                        match crate::ws_connections::messages::ws2p_recv_message_pretreatment(
                            &mut self, msg,
                        ) {
                            WS2PSignal::NoConnection => {
                                warn!("WS2PSignal::NoConnection");
                            }
                            WS2PSignal::ConnectionEstablished(ws2p_full_id) => {
                                let module_req_id =
                                    ModuleReqId(self.requests_awaiting_response.len() as u32);
                                let module_id = WS2Pv1Module::name();
                                debug!("WS2P: send req to: ({:?})", ws2p_full_id);
                                let _current_request_result =
                                    ws_connections::requests::sent::send_request_to_specific_node(
                                        &mut self,
                                        ModuleReqFullId(module_id, module_req_id),
                                        &ws2p_full_id,
                                        &WS2Pv1Request {
                                            id: WS2Pv1ReqId::random(),
                                            body: WS2Pv1ReqBody::GetCurrent,
                                        },
                                    );
                                if self.uids_cache.get(&ws2p_full_id.1).is_none() {
                                    send_dal_request(
                                        &mut self,
                                        &BlockchainRequest::UIDs(vec![ws2p_full_id.1]),
                                    );
                                }
                                let event = NetworkEvent::ConnectionStateChange(
                                    ws2p_full_id,
                                    WS2PConnectionState::Established as u32,
                                    self.uids_cache.get(&ws2p_full_id.1).cloned(),
                                    self.ws2p_endpoints[&ws2p_full_id]
                                        .ep
                                        .get_url(false, false)
                                        .expect("Endpoint unreachable !"),
                                );
                                events::sent::send_network_event(&mut self, event);
                            }
                            WS2PSignal::WSError(ws2p_full_id) => {
                                endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                                close_connection(
                                    &mut self,
                                    &ws2p_full_id,
                                    WS2PCloseConnectionReason::WsError,
                                );
                                let event = NetworkEvent::ConnectionStateChange(
                                    ws2p_full_id,
                                    WS2PConnectionState::WSError as u32,
                                    self.uids_cache.get(&ws2p_full_id.1).cloned(),
                                    self.ws2p_endpoints[&ws2p_full_id]
                                        .ep
                                        .get_url(false, false)
                                        .expect("Endpoint unreachable !"),
                                );
                                events::sent::send_network_event(&mut self, event);
                            }
                            WS2PSignal::NegociationTimeout(ws2p_full_id) => {
                                endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                                let event = NetworkEvent::ConnectionStateChange(
                                    ws2p_full_id,
                                    WS2PConnectionState::Denial as u32,
                                    self.uids_cache.get(&ws2p_full_id.1).cloned(),
                                    self.ws2p_endpoints[&ws2p_full_id]
                                        .ep
                                        .get_url(false, false)
                                        .expect("Endpoint unreachable !"),
                                );
                                events::sent::send_network_event(&mut self, event);
                            }
                            WS2PSignal::Timeout(ws2p_full_id) => {
                                endpoints_to_update_status.insert(ws2p_full_id, SystemTime::now());
                                let event = NetworkEvent::ConnectionStateChange(
                                    ws2p_full_id,
                                    WS2PConnectionState::Close as u32,
                                    self.uids_cache.get(&ws2p_full_id.1).cloned(),
                                    self.ws2p_endpoints[&ws2p_full_id]
                                        .ep
                                        .get_url(false, false)
                                        .expect("Endpoint unreachable !"),
                                );
                                events::sent::send_network_event(&mut self, event);
                            }
                            WS2PSignal::PeerCard(_ws2p_full_id, _peer_card, ws2p_endpoints) => {
                                //trace!("WS2PSignal::PeerCard({})", ws2p_full_id);
                                //self.send_network_event(NetworkEvent::ReceivePeers(_));
                                for ep in ws2p_endpoints {
                                    match self.ws2p_endpoints.get(
                                        &ep.node_full_id()
                                            .expect("WS2P: Fail to get ep.node_full_id() !"),
                                    ) {
                                        Some(_) => {}
                                        None => {
                                            if let Some(_api) =
                                                ws2p_db::string_to_api(&ep.api.0.clone())
                                            {
                                                endpoints_to_update_status.insert(
                                                    ep.node_full_id().expect(
                                                        "WS2P: Fail to get ep.node_full_id() !",
                                                    ),
                                                    SystemTime::now(),
                                                );
                                            }
                                            if cfg!(feature = "ssl") || ep.port != 443 {
                                                connect_to(&mut self, &ep);
                                            }
                                        }
                                    };
                                }
                            }
                            WS2PSignal::Heads(ws2p_full_id, heads) => {
                                trace!("WS2PSignal::Heads({}, {:?})", ws2p_full_id, heads.len());
                                send_dal_request(
                                    &mut self,
                                    &BlockchainRequest::UIDs(
                                        heads.iter().map(NetworkHead::pubkey).collect(),
                                    ),
                                );
                                let event = NetworkEvent::ReceiveHeads(
                                    heads
                                        .iter()
                                        .map(|head| {
                                            let mut new_head = head.clone();
                                            if let Some(uid) = self.uids_cache.get(&head.pubkey()) {
                                                new_head.set_uid(uid);
                                            }
                                            new_head
                                        })
                                        .collect(),
                                );
                                events::sent::send_network_event(&mut self, event);
                            }
                            WS2PSignal::Blocks(ws2p_full_id, blocks) => {
                                trace!("WS2PSignal::Blocks({})", ws2p_full_id);
                                events::sent::send_network_event(
                                    &mut self,
                                    NetworkEvent::ReceiveBlocks(blocks),
                                );
                            }
                            WS2PSignal::UserDocuments(ws2p_full_id, user_documents) => {
                                trace!("WS2PSignal::UserDocuments({})", ws2p_full_id);
                                events::sent::send_network_event(
                                    &mut self,
                                    NetworkEvent::ReceiveDocuments(user_documents),
                                );
                            }
                            WS2PSignal::Request { from, req_id, body } => {
                                ws_connections::requests::received::receive_ws2p_v1_request(
                                    &mut self, from, req_id, body,
                                );
                            }
                            WS2PSignal::ReqResponse(
                                module_req_full_id,
                                ws2p_req_body,
                                recipient_full_id,
                                response,
                            ) => ws_connections::responses::received::receive_response(
                                &mut self,
                                module_req_full_id,
                                ws2p_req_body,
                                recipient_full_id,
                                response,
                            ),
                            WS2PSignal::Empty => {}
                        }
                    }
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Disconnected => {
                        fatal_error!("Disconnected ws2p module !");
                    }
                    mpsc::RecvTimeoutError::Timeout => {}
                },
            }
            if unwrap!(SystemTime::now().duration_since(last_ws2p_endpoints_write))
                > Duration::new(*DURATION_BETWEEN_2_ENDPOINTS_SAVING, 0)
            {
                last_ws2p_endpoints_write = SystemTime::now();
                if let Err(err) = ws2p_db::write_endpoints(&self.ep_file_path, &self.ws2p_endpoints)
                {
                    fatal_error!("WS2P1: Fail to write endpoints in DB : {:?}", err);
                }
            }
            if unwrap!(SystemTime::now().duration_since(last_ws2p_state_print))
                > Duration::new(*WS2P_GENERAL_STATE_INTERVAL, 0)
            {
                last_ws2p_state_print = SystemTime::now();
                let mut connected_nodes = Vec::new();
                for (k, DbEndpoint { state, .. }) in self.ws2p_endpoints.clone() {
                    if let WS2PConnectionState::Established = state {
                        connected_nodes.push(k);
                    }
                }
                // Print current_blockstamp
                info!(
                    "WS2Pv1Module : current_blockstamp() = {:?}",
                    self.current_blockstamp
                );
                // New WS2P connection wave
                if connected_nodes.len() < self.conf.clone().outcoming_quota
                    && (unwrap!(SystemTime::now().duration_since(last_ws2p_connecting_wave))
                        > Duration::new(*WS2P_OUTCOMING_INTERVAL, 0)
                        || (unwrap!(SystemTime::now().duration_since(last_ws2p_connecting_wave))
                            > Duration::new(*WS2P_OUTCOMING_INTERVAL_AT_STARTUP, 0)
                            && unwrap!(SystemTime::now().duration_since(start_time))
                                < Duration::new(*WS2P_OUTCOMING_INTERVAL, 0)))
                {
                    last_ws2p_connecting_wave = SystemTime::now();
                    info!("Connected to know endpoints...");
                    connect_to_know_endpoints(&mut self);
                }
                // Request pending_identities from network
                if unwrap!(SystemTime::now().duration_since(last_identities_request))
                    > Duration::new(*PENDING_IDENTITIES_REQUEST_INTERVAL, 0)
                    && unwrap!(SystemTime::now().duration_since(start_time)) > Duration::new(10, 0)
                {
                    /*info!("get pending_identities from all connections...");
                    let _blocks_request_result = self.send_request_to_all_connections(
                        &OldNetworkRequest::GetRequirementsPending(ModuleReqId(0 as u32), 5),
                    );*/
                    last_identities_request = SystemTime::now();
                }
                // ..
                // Request current blockstamp
                send_dal_request(&mut self, &BlockchainRequest::CurrentBlockstamp());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws_connections::requests::sent::network_request_to_json;
    use crate::ws_connections::requests::*;
    use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
    use dubp_block_doc::parser::parse_json_block_from_serde_value;
    use dubp_common_doc::BlockNumber;

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
        let block: BlockDocument = parse_json_block_from_serde_value(&json_block)
            .expect("Fail to parse test json block !");
        assert_eq!(
            block
                .inner_hash()
                .expect("Try to get inner_hash of an uncompleted or reduce block !")
                .to_hex(),
            "61F02B1A6AE2E4B9A1FD66CE673258B4B21C0076795571EE3C9DC440DD06C46C"
        );
        //block.generate_hash();
        assert_eq!(
            block
                .hash()
                .expect("Try to get hash of an uncompleted or reduce block !")
                .0
                .to_hex(),
            "000000EF5B2AA849F4C3AF3D35E1284EA1F34A9F617EA806CE8371619023DC74"
        );
    }

    #[test]
    fn ws2p_requests() {
        let req_id_str = "fbcf0bfa-7e18-40cc-b300-5c797d27518e";
        let req_id = WS2Pv1ReqId::from_str(req_id_str).expect("fail to parse req_id");
        let request = WS2Pv1Request {
            id: req_id,
            body: WS2Pv1ReqBody::GetBlocks {
                count: 50,
                from_number: BlockNumber(0),
            },
        };
        assert_eq!(
            network_request_to_json(&request),
            json!({
                "reqId": req_id_str,
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
            "{\"body\":{\"name\":\"BLOCKS_CHUNK\",\"params\":{\"count\":50,\"fromNumber\":0}},\"reqId\":\"fbcf0bfa-7e18-40cc-b300-5c797d27518e\"}"
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
            fatal_error!("Fail to parse head !")
        }
        assert_eq!(heads_count, 1);
    }
}

extern crate serde_json;
extern crate websocket;

use duniter_crypto::keys::*;
use duniter_dal::parsers::blocks::parse_json_block;
use duniter_module::ModuleReqId;
use duniter_network::network_endpoint::{NetworkEndpoint, NetworkEndpointApi};
use duniter_network::{NetworkDocument, NodeUUID};
use std::fmt::Debug;
use std::net::TcpStream;

use super::{NodeFullId, WS2PAckMessageV1, WS2PConnectMessageV1, WS2PMessage, WS2POkMessageV1};

#[derive(Debug, Copy, Clone)]
pub enum WS2POrderForListeningThread {
    Close,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WS2PConnectionState {
    NeverTry = 0,
    TryToOpenWS = 1,
    WSError = 2,
    TryToSendConnectMess = 3,
    Unreachable = 4,
    WaitingConnectMess = 5,
    NoResponse = 6,
    ConnectMessOk = 7,
    OkMessOkWaitingAckMess = 8,
    AckMessOk = 9,
    Denial = 10,
    Close = 11,
    Established = 12,
}

impl From<u32> for WS2PConnectionState {
    fn from(integer: u32) -> Self {
        match integer {
            1 | 2 => WS2PConnectionState::WSError,
            3 | 4 => WS2PConnectionState::Unreachable,
            5 | 6 => WS2PConnectionState::NoResponse,
            7 | 8 | 9 | 10 => WS2PConnectionState::Denial,
            11 | 12 => WS2PConnectionState::Close,
            _ => WS2PConnectionState::NeverTry,
        }
    }
}

impl WS2PConnectionState {
    pub fn from_u32(integer: u32, from_db: bool) -> Self {
        if from_db {
            WS2PConnectionState::from(integer)
        } else {
            match integer {
                1 => WS2PConnectionState::TryToOpenWS,
                2 => WS2PConnectionState::WSError,
                3 | 4 => WS2PConnectionState::Unreachable,
                5 | 6 => WS2PConnectionState::NoResponse,
                7 => WS2PConnectionState::ConnectMessOk,
                8 => WS2PConnectionState::OkMessOkWaitingAckMess,
                9 => WS2PConnectionState::AckMessOk,
                10 => WS2PConnectionState::Denial,
                11 => WS2PConnectionState::Close,
                12 => WS2PConnectionState::Established,
                _ => WS2PConnectionState::NeverTry,
            }
        }
    }
    pub fn to_u32(&self) -> u32 {
        match *self {
            WS2PConnectionState::NeverTry => 0,
            _ => 1,
        }
    }
}

pub struct WebsocketSender(pub websocket::sender::Writer<TcpStream>);

impl Debug for WebsocketSender {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "WebsocketSender {{ }}")
    }
}

#[derive(Debug)]
pub enum WS2PConnectionMessagePayload {
    FailOpenWS,
    WrongUrl,
    FailToSplitWS,
    TryToSendConnectMess,
    FailSendConnectMess,
    WebsocketOk(WebsocketSender),
    NegociationTimeout,
    ValidConnectMessage(String, WS2PConnectionState),
    ValidAckMessage(String, WS2PConnectionState),
    ValidOk(WS2PConnectionState),
    DalRequest(ModuleReqId, serde_json::Value),
    PeerCard(serde_json::Value, Vec<NetworkEndpoint>),
    Heads(Vec<serde_json::Value>),
    Document(NetworkDocument),
    ReqResponse(ModuleReqId, serde_json::Value),
    InvalidMessage,
    WrongFormatMessage,
    UnknowMessage,
    Timeout,
    Close,
}

#[derive(Debug)]
pub struct WS2PConnectionMessage(pub NodeFullId, pub WS2PConnectionMessagePayload);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WS2PCloseConnectionReason {
    AuthMessInvalidSig,
    NegociationTimeout,
    Timeout,
    Unknow,
}

#[derive(Debug, Clone)]
pub struct WS2PConnectionMetaData {
    pub state: WS2PConnectionState,
    pub remote_uuid: Option<NodeUUID>,
    pub remote_pubkey: Option<PubKey>,
    pub challenge: String,
    pub remote_challenge: String,
    pub current_blockstamp: Option<(u32, String)>,
}

#[derive(Debug, Clone)]
pub struct WS2PDatasForListeningThread {
    pub conn_meta_datas: WS2PConnectionMetaData,
    pub currency: String,
    pub key_pair: KeyPairEnum,
}

impl WS2PConnectionMetaData {
    pub fn new(challenge: String) -> Self {
        WS2PConnectionMetaData {
            state: WS2PConnectionState::WaitingConnectMess,
            remote_uuid: None,
            remote_pubkey: None,
            challenge,
            remote_challenge: "".to_string(),
            current_blockstamp: None,
        }
    }

    pub fn node_full_id(&self) -> NodeFullId {
        NodeFullId(
            self.clone()
                .remote_uuid
                .expect("Fail to get NodeFullId : remote_uuid is None !"),
            self.remote_pubkey
                .expect("Fail to get NodeFullId : remote_pubkey is None !"),
        )
    }
    pub fn parse_and_check_incoming_message(
        &mut self,
        currency: &str,
        key_pair: KeyPairEnum,
        m: &serde_json::Value,
    ) -> WS2PConnectionMessagePayload {
        if let Some(s) = m.get("auth") {
            if s.is_string() {
                match s.as_str().unwrap() {
                    "CONNECT" => {
                        let message = WS2PConnectMessageV1::parse(m, currency.to_string())
                            .expect("Failed to parsing CONNECT Message !");
                        if message.verify() && message.pubkey == self.remote_pubkey.unwrap() {
                            match self.state {
                                WS2PConnectionState::WaitingConnectMess => {
                                    trace!("CONNECT sig is valid.");
                                    self.state = WS2PConnectionState::ConnectMessOk;
                                    self.remote_challenge = message.challenge.clone();
                                    let mut response = WS2PAckMessageV1 {
                                        currency: currency.to_string(),
                                        pubkey: key_pair.public_key(),
                                        challenge: self.remote_challenge.clone(),
                                        signature: None,
                                    };
                                    response.signature = Some(response.sign(key_pair));
                                    return WS2PConnectionMessagePayload::ValidConnectMessage(
                                        serde_json::to_string(&response).unwrap(),
                                        self.state,
                                    );
                                }
                                _ => return WS2PConnectionMessagePayload::InvalidMessage,
                            }
                        } else {
                            warn!("The signature of message CONNECT is invalid !")
                        }
                    }
                    "ACK" => {
                        let mut message = WS2PAckMessageV1::parse(m, currency.to_string())
                            .expect("Failed to parsing ACK Message !");
                        message.challenge = self.challenge.to_string();
                        if message.verify() {
                            trace!("ACK sig is valid.");
                            self.state = match self.state {
                                WS2PConnectionState::ConnectMessOk => {
                                    WS2PConnectionState::AckMessOk
                                }
                                WS2PConnectionState::OkMessOkWaitingAckMess => {
                                    WS2PConnectionState::Established
                                }
                                _ => return WS2PConnectionMessagePayload::InvalidMessage,
                            };
                            let mut response = WS2POkMessageV1 {
                                currency: currency.to_string(),
                                pubkey: key_pair.public_key(),
                                challenge: self.challenge.to_string(),
                                signature: None,
                            };
                            response.signature = Some(response.sign(key_pair));
                            return WS2PConnectionMessagePayload::ValidAckMessage(
                                serde_json::to_string(&response).unwrap(),
                                self.state,
                            );
                        } else {
                            warn!("The signature of message ACK is invalid !")
                        }
                    }
                    "OK" => {
                        let mut message = WS2POkMessageV1::parse(m, currency.to_string())
                            .expect("Failed to parsing OK Message !");
                        trace!("Received OK");
                        message.challenge = self.remote_challenge.to_string();
                        message.pubkey = self.remote_pubkey.expect("fail to get remote pubkey !");
                        if message.verify() {
                            trace!("OK sig is valid.");
                            match self.state {
                                WS2PConnectionState::ConnectMessOk => {
                                    self.state = WS2PConnectionState::OkMessOkWaitingAckMess;
                                    return WS2PConnectionMessagePayload::ValidOk(self.state);
                                }
                                WS2PConnectionState::AckMessOk => {
                                    info!(
                                        "WS2P Connection established with the key {}",
                                        self.remote_pubkey.expect("fail to get remote pubkey !")
                                    );
                                    self.state = WS2PConnectionState::Established;
                                    return WS2PConnectionMessagePayload::ValidOk(self.state);
                                }
                                _ => {
                                    warn!("WS2P Error : OK message not expected !");
                                    return WS2PConnectionMessagePayload::InvalidMessage;
                                }
                            }
                        } else {
                            warn!("The signature of message OK is invalid !");
                            return WS2PConnectionMessagePayload::InvalidMessage;
                        }
                    }
                    &_ => debug!("unknow message"),
                };
            }
        };
        if let Some(req_id) = m.get("reqId") {
            match req_id.as_str() {
                Some(req_id) => match m.get("body") {
                    Some(body) => {
                        trace!("WS2P : Receive DAL Request from {}.", self.node_full_id());
                        match u32::from_str_radix(req_id, 16) {
                            Ok(req_id) => {
                                return WS2PConnectionMessagePayload::DalRequest(
                                    ModuleReqId(req_id),
                                    body.clone(),
                                );
                            }
                            Err(_) => return WS2PConnectionMessagePayload::WrongFormatMessage,
                        }
                    }
                    None => {
                        warn!("WS2P Error : invalid format : Request must contain a field body !");
                        return WS2PConnectionMessagePayload::WrongFormatMessage;
                    }
                },
                None => {
                    warn!("WS2P Error : invalid format : Request must contain a field body !");
                    return WS2PConnectionMessagePayload::WrongFormatMessage;
                }
            }
        }
        if let Some(req_id) = m.get("resId") {
            match req_id.as_str() {
                Some(req_id_str) => match m.get("body") {
                    Some(body) => match u32::from_str_radix(req_id_str, 16) {
                        Ok(req_id) => {
                            return WS2PConnectionMessagePayload::ReqResponse(
                                ModuleReqId(req_id),
                                body.clone(),
                            )
                        }
                        Err(_) => return WS2PConnectionMessagePayload::WrongFormatMessage,
                    },
                    None => match m.get("err") {
                        Some(err) => warn!("Error in req : {:?}", err),
                        None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                    },
                },
                None => return WS2PConnectionMessagePayload::WrongFormatMessage,
            }
        }
        if let Some(body) = m.get("body") {
            match body.get("name") {
                Some(s) => if s.is_string() {
                    match s.as_str().unwrap() {
                        "BLOCK" => match body.get("block") {
                            Some(block) => {
                                if let Some(network_block) = parse_json_block(&block) {
                                    return WS2PConnectionMessagePayload::Document(
                                        NetworkDocument::Block(network_block),
                                    );
                                } else {
                                    info!("WS2PSignal: receive invalid block (wrong format).");
                                };
                            }
                            None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                        },
                        "HEAD" => match body.get("heads") {
                            Some(heads) => match heads.as_array() {
                                Some(heads_array) => {
                                    return WS2PConnectionMessagePayload::Heads(heads_array.clone())
                                }
                                None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                            },
                            None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                        },
                        "PEER" => return self.parse_and_check_peer_message(body),
                        "CERTIFICATION" => {
                            trace!("WS2P : Receive CERTIFICATION from {}.", self.node_full_id());
                            /*return WS2PConnectionMessagePayload::Document(
                                NetworkDocument::Certification(_)
                            );*/
                        }
                        _ => {
                            /*trace!(
                                "WS2P : Receive Unknow Message from {}.",
                                self.node_full_id()
                            );*/
                            return WS2PConnectionMessagePayload::UnknowMessage;
                        }
                    };
                },
                None => {
                    warn!("WS2P Error : invalid format : Body must contain a field name !");
                    return WS2PConnectionMessagePayload::WrongFormatMessage;
                }
            }
        };
        WS2PConnectionMessagePayload::UnknowMessage
    }

    pub fn parse_and_check_peer_message(
        &mut self,
        body: &serde_json::Value,
    ) -> WS2PConnectionMessagePayload {
        match body.get("peer") {
            Some(peer) => match peer.get("pubkey") {
                Some(raw_pubkey) => match ed25519::PublicKey::from_base58(
                    raw_pubkey.as_str().unwrap_or(""),
                ) {
                    Ok(pubkey) => {
                        let mut ws2p_endpoints: Vec<NetworkEndpoint> = Vec::new();
                        match peer.get("endpoints") {
                            Some(endpoints) => match endpoints.as_array() {
                                Some(array_endpoints) => {
                                    for endpoint in array_endpoints {
                                        if let Some(ep) = NetworkEndpoint::parse_from_raw(
                                            endpoint.as_str().unwrap_or(""),
                                            PubKey::Ed25519(pubkey),
                                            0,
                                            0,
                                        ) {
                                            if ep.api() == NetworkEndpointApi(String::from("WS2P"))
                                            {
                                                ws2p_endpoints.push(ep);
                                            }
                                        }
                                    }
                                    WS2PConnectionMessagePayload::PeerCard(
                                        body.clone(),
                                        ws2p_endpoints,
                                    )
                                }
                                None => WS2PConnectionMessagePayload::WrongFormatMessage,
                            },
                            None => WS2PConnectionMessagePayload::WrongFormatMessage,
                        }
                    }
                    Err(_) => WS2PConnectionMessagePayload::WrongFormatMessage,
                },
                None => WS2PConnectionMessagePayload::WrongFormatMessage,
            },
            None => WS2PConnectionMessagePayload::WrongFormatMessage,
        }
    }
}

use constants::*;
use duniter_module::ModuleReqId;
use duniter_network::BlockchainDocument;
use dup_crypto::keys::*;
use durs_network_documents::network_endpoint::{EndpointEnum, NetworkEndpointApi};
use durs_network_documents::NodeId;
use parsers::blocks::parse_json_block;
use rand::Rng;
use std::sync::mpsc;
use ws::deflate::DeflateBuilder;
#[allow(deprecated)]
use ws::util::{Timeout, Token};
use ws::{connect, CloseCode, Frame, Handler, Handshake, Message, Sender};
use *;

const CONNECT: Token = Token(1);
const EXPIRE: Token = Token(2);

/// Store a websocket sender
pub struct WsSender(pub Sender);

impl ::std::fmt::Debug for WsSender {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "WsSender {{ }}")
    }
}

// Our Handler struct.
// Here we explicity indicate that the Client needs a Sender,
// whereas a closure captures the Sender for us automatically.
#[allow(deprecated)]
struct Client {
    ws: Sender,
    conductor_sender: mpsc::Sender<WS2PThreadSignal>,
    currency: String,
    key_pair: KeyPairEnum,
    connect_message: Message,
    conn_meta_datas: WS2PConnectionMetaDatas,
    last_mess_time: SystemTime,
    spam_interval: bool,
    spam_counter: usize,
    timeout: Option<Timeout>,
}

// We implement the Handler trait for Client so that we can get more
// fine-grained control of the connection.
impl Handler for Client {
    // `on_open` will be called only after the WebSocket handshake is successful
    // so at this point we know that the connection is ready to send/receive messages.
    // We ignore the `Handshake` for now, but you could also use this method to setup
    // Handler state or reject the connection based on the details of the Request
    // or Response, such as by checking cookies or Auth headers.
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        // Define timeouts
        self.ws.timeout(WS2P_NEGOTIATION_TIMEOUT * 1_000, CONNECT)?;
        self.ws.timeout(WS2P_EXPIRE_TIMEOUT * 1_000, EXPIRE)?;
        // Send ws::Sender to WS2PConductor
        let result = self
            .conductor_sender
            .send(WS2PThreadSignal::WS2PConnectionMessage(
                WS2PConnectionMessage(
                    self.conn_meta_datas.node_full_id(),
                    WS2PConnectionMessagePayload::WebsocketOk(WsSender(self.ws.clone())),
                ),
            ));
        // If WS2PConductor is unrechable, close connection.
        if result.is_err() {
            debug!("Close ws2p connection because ws2p main thread is unrechable !");
            self.ws.close(CloseCode::Normal)
        } else {
            // Send CONNECT Message
            self.ws.send(self.connect_message.clone())
        }
    }

    // `on_message` is roughly equivalent to the Handler closure. It takes a `Message`
    // and returns a `Result<()>`.
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        // Spam ?
        if SystemTime::now()
            .duration_since(self.last_mess_time)
            .unwrap()
            > Duration::new(*WS2P_SPAM_INTERVAL_IN_MILLI_SECS, 0)
        {
            if self.spam_interval {
                self.spam_counter += 1;
            } else {
                self.spam_interval = true;
                self.spam_counter = 2;
            }
        } else {
            self.spam_interval = false;
            self.spam_counter = 0;
        }
        // Spam ?
        if self.spam_counter >= *WS2P_SPAM_LIMIT {
            thread::sleep(Duration::from_millis(*WS2P_SPAM_SLEEP_TIME_IN_SEC));
            self.last_mess_time = SystemTime::now();
            return Ok(());
        }
        self.last_mess_time = SystemTime::now();

        // Parse and check incoming message
        if msg.is_text() {
            let s: String = msg
                .into_text()
                .expect("WS2P: Fail to convert message payload to String !");
            trace!("WS2P: receive mess: {}", s);
            let json_message: serde_json::Value = serde_json::from_str(&s)
                .expect("WS2P: Fail to convert string message ton json value !");
            let result = self
                .conductor_sender
                .send(WS2PThreadSignal::WS2PConnectionMessage(
                    WS2PConnectionMessage(
                        self.conn_meta_datas.node_full_id(),
                        self.conn_meta_datas.parse_and_check_incoming_message(
                            &self.currency,
                            self.key_pair,
                            &json_message,
                        ),
                    ),
                ));
            if result.is_err() {
                info!("Close ws2p connection because ws2p main thread is unrechable !");
                self.ws.close(CloseCode::Normal)?;
            }
        }
        Ok(())
    }
    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        match event {
            CONNECT => {
                if self.conn_meta_datas.state != WS2PConnectionState::Established {
                    let _result =
                        self.conductor_sender
                            .send(WS2PThreadSignal::WS2PConnectionMessage(
                                WS2PConnectionMessage(
                                    self.conn_meta_datas.node_full_id(),
                                    WS2PConnectionMessagePayload::NegociationTimeout,
                                ),
                            ));
                    self.ws.close(CloseCode::Away)
                } else {
                    Ok(())
                }
            }
            EXPIRE => {
                let _result = self
                    .conductor_sender
                    .send(WS2PThreadSignal::WS2PConnectionMessage(
                        WS2PConnectionMessage(
                            self.conn_meta_datas.node_full_id(),
                            WS2PConnectionMessagePayload::Timeout,
                        ),
                    ));
                self.ws.close(CloseCode::Away)
            }
            _ => Ok(()),
        }
    }
    #[allow(deprecated)]
    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> ws::Result<()> {
        if event == EXPIRE {
            if let Some(t) = self.timeout.take() {
                self.ws.cancel(t)?;
            }
            self.timeout = Some(timeout)
        }
        Ok(())
    }
    fn on_frame(&mut self, frame: Frame) -> ws::Result<Option<Frame>> {
        // some activity has occurred, let's reset the expiration timeout
        self.ws.timeout(WS2P_EXPIRE_TIMEOUT * 1_000, EXPIRE)?;
        Ok(Some(frame))
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        match code {
            CloseCode::Normal => info!("The remote server close the connection."),
            CloseCode::Away => info!("The remote server is leaving."),
            _ => warn!("The remote server encountered an error: {}", reason),
        }
        let _result = self
            .conductor_sender
            .send(WS2PThreadSignal::WS2PConnectionMessage(
                WS2PConnectionMessage(
                    self.conn_meta_datas.node_full_id(),
                    WS2PConnectionMessagePayload::Close,
                ),
            ));
    }
}

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
    pub fn to_u32(self) -> u32 {
        match self {
            WS2PConnectionState::NeverTry => 0,
            _ => 1,
        }
    }
}

#[derive(Debug)]
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
    PeerCard(serde_json::Value, Vec<EndpointEnum>),
    Heads(Vec<serde_json::Value>),
    Document(BlockchainDocument),
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
pub struct WS2PConnectionMetaDatas {
    pub state: WS2PConnectionState,
    pub remote_uuid: Option<NodeId>,
    pub remote_pubkey: Option<PubKey>,
    pub challenge: String,
    pub remote_challenge: String,
    pub current_blockstamp: Option<(u32, String)>,
}

#[derive(Debug, Clone)]
pub struct WS2PDatasForListeningThread {
    pub conn_meta_datas: WS2PConnectionMetaDatas,
    pub currency: String,
    pub key_pair: KeyPairEnum,
}

impl WS2PConnectionMetaDatas {
    pub fn new(challenge: String) -> Self {
        WS2PConnectionMetaDatas {
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
                                    debug!("CONNECT sig is valid.");
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
                Some(s) => {
                    if s.is_string() {
                        match s.as_str().unwrap() {
                            "BLOCK" => match body.get("block") {
                                Some(block) => {
                                    if let Some(network_block) = parse_json_block(&block) {
                                        return WS2PConnectionMessagePayload::Document(
                                            BlockchainDocument::Block(network_block),
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
                                        return WS2PConnectionMessagePayload::Heads(
                                            heads_array.clone(),
                                        )
                                    }
                                    None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                                },
                                None => return WS2PConnectionMessagePayload::WrongFormatMessage,
                            },
                            "PEER" => return self.parse_and_check_peer_message(body),
                            "CERTIFICATION" => {
                                trace!(
                                    "WS2P : Receive CERTIFICATION from {}.",
                                    self.node_full_id()
                                );
                                /*return WS2PConnectionMessagePayload::Document(
                                    BlockchainDocument::Certification(_)
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
                    }
                }
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
                Some(raw_pubkey) => {
                    match ed25519::PublicKey::from_base58(raw_pubkey.as_str().unwrap_or("")) {
                        Ok(pubkey) => {
                            let mut ws2p_endpoints: Vec<EndpointEnum> = Vec::new();
                            match peer.get("endpoints") {
                                Some(endpoints) => match endpoints.as_array() {
                                    Some(array_endpoints) => {
                                        for endpoint in array_endpoints {
                                            if let Ok(ep) = EndpointEnum::parse_from_raw(
                                                endpoint.as_str().unwrap_or(""),
                                                PubKey::Ed25519(pubkey),
                                                0,
                                                0,
                                                1u16,
                                            ) {
                                                if ep.api()
                                                    == NetworkEndpointApi(String::from("WS2P"))
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
                    }
                }
                None => WS2PConnectionMessagePayload::WrongFormatMessage,
            },
            None => WS2PConnectionMessagePayload::WrongFormatMessage,
        }
    }
}

pub fn get_random_connection<S: ::std::hash::BuildHasher>(
    connections: &HashMap<NodeFullId, (EndpointEnum, WS2PConnectionState), S>,
) -> NodeFullId {
    let mut rng = rand::thread_rng();
    let mut loop_count = 0;
    loop {
        for (ws2p_full_id, (_ep, state)) in &(*connections) {
            if loop_count > 10 {
                return *ws2p_full_id;
            }
            if let WS2PConnectionState::Established = state {
                if rng.gen::<bool>() {
                    return *ws2p_full_id;
                }
            }
        }
        loop_count += 1;
    }
}

pub fn connect_to_ws2p_endpoint(
    endpoint: &EndpointEnum,
    conductor_sender: &mpsc::Sender<WS2PThreadSignal>,
    currency: &str,
    key_pair: KeyPairEnum,
) -> ws::Result<()> {
    // Get endpoint url
    let ws_url = endpoint.get_url(true, false).expect("Endpoint unreachable");

    // Create WS2PConnectionMetaDatass
    let mut conn_meta_datas = WS2PConnectionMetaDatas::new(
        "b60a14fd-0826-4ae0-83eb-1a92cd59fd5308535fd3-78f2-4678-9315-cd6e3b7871b1".to_string(),
    );
    conn_meta_datas.remote_pubkey = Some(endpoint.pubkey());
    conn_meta_datas.remote_uuid = Some(
        endpoint
            .node_uuid()
            .expect("WS2P: Fail to get ep.node_uuid() !"),
    );

    // Generate connect message
    let connect_message =
        generate_connect_message(currency, key_pair, conn_meta_datas.challenge.clone());

    // Log
    info!("Try connection to {} ...", ws_url);

    // Connect to websocket
    connect(ws_url, |ws| {
        DeflateBuilder::new().build(Client {
            ws,
            conductor_sender: conductor_sender.clone(),
            currency: String::from(currency),
            key_pair,
            connect_message: connect_message.clone(),
            conn_meta_datas: conn_meta_datas.clone(),
            last_mess_time: SystemTime::now(),
            spam_interval: false,
            spam_counter: 0,
            timeout: None,
        })
    })
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

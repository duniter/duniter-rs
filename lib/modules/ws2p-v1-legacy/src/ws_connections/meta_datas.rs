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

//! WS2P connections meta datas.

use super::messages::WS2PConnectionMessagePayload;
use super::states::WS2PConnectionState;
use crate::parsers::blocks::parse_json_block;
use crate::*;
use duniter_network::documents::BlockchainDocument;
use dup_crypto::keys::*;
use durs_module::ModuleReqId;
use durs_network_documents::network_endpoint::{EndpointV1, NetworkEndpointApi};
use durs_network_documents::NodeId;
#[allow(deprecated)]
#[derive(Debug, Clone)]
pub struct WS2PConnectionMetaDatas {
    pub state: WS2PConnectionState,
    pub remote_uuid: Option<NodeId>,
    pub remote_pubkey: Option<PubKey>,
    pub challenge: String,
    pub remote_challenge: String,
    pub current_blockstamp: Option<(u32, String)>,
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
                match s.as_str().unwrap_or("") {
                    "CONNECT" => {
                        let message = WS2PConnectMessageV1::parse(m, currency.to_string())
                            .expect("Failed to parsing CONNECT Message !");
                        if message.verify() && message.pubkey == unwrap!(self.remote_pubkey) {
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
                                        unwrap!(serde_json::to_string(&response)),
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
                                unwrap!(serde_json::to_string(&response)),
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
                            );
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
                        match s.as_str().unwrap_or("") {
                            "BLOCK" => match body.get("block") {
                                Some(block) => {
                                    if let Some(block_doc) = parse_json_block(&block) {
                                        return WS2PConnectionMessagePayload::Document(
                                            BlockchainDocument::Block(Box::new(block_doc)),
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
                                        );
                                    }
                                    None => {
                                        return WS2PConnectionMessagePayload::WrongFormatMessage
                                    }
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
                            let mut ws2p_endpoints: Vec<EndpointV1> = Vec::new();
                            match peer.get("endpoints") {
                                Some(endpoints) => match endpoints.as_array() {
                                    Some(array_endpoints) => {
                                        for endpoint in array_endpoints {
                                            if let Ok(ep) = EndpointV1::parse_from_raw(
                                                endpoint.as_str().unwrap_or(""),
                                                PubKey::Ed25519(pubkey),
                                                0,
                                                0,
                                            ) {
                                                if ep.api
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

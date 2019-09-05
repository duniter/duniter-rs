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

//! WS2P connections meta datas.

use super::messages::WS2Pv1MsgPayload;
use super::states::WS2PConnectionState;
use crate::ws_connections::requests::{WS2Pv1ReqBody, WS2Pv1ReqId};
use crate::*;
use dubp_block_doc::parser::parse_json_block_from_serde_value;
use dubp_block_doc::DocumentDUBP;
use dup_crypto::keys::*;
use durs_network_documents::network_endpoint::{ApiName, EndpointV1};
use durs_network_documents::NodeId;
use std::convert::TryFrom;

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
        signator: &SignatorEnum,
        msg: &serde_json::Value,
    ) -> WS2Pv1MsgPayload {
        if let Some(s) = msg.get("auth") {
            if s.is_string() {
                match s.as_str().unwrap_or("") {
                    "CONNECT" => {
                        let message = WS2PConnectMessageV1::parse(msg, currency.to_string())
                            .expect("Failed to parsing CONNECT Message !");
                        if message.verify() && message.pubkey == unwrap!(self.remote_pubkey) {
                            match self.state {
                                WS2PConnectionState::WaitingConnectMess => {
                                    debug!("CONNECT sig is valid.");
                                    self.state = WS2PConnectionState::ConnectMessOk;
                                    self.remote_challenge = message.challenge.clone();
                                    let mut response = WS2PAckMessageV1 {
                                        currency: currency.to_string(),
                                        pubkey: signator.public_key(),
                                        challenge: self.remote_challenge.clone(),
                                        signature: None,
                                    };
                                    response.signature = Some(response.sign(signator));
                                    return WS2Pv1MsgPayload::ValidConnectMessage(
                                        unwrap!(serde_json::to_string(&response)),
                                        self.state,
                                    );
                                }
                                _ => return WS2Pv1MsgPayload::InvalidMessage,
                            }
                        } else {
                            warn!("The signature of message CONNECT is invalid !")
                        }
                    }
                    "ACK" => {
                        let mut message = WS2PAckMessageV1::parse(msg, currency.to_string())
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
                                _ => return WS2Pv1MsgPayload::InvalidMessage,
                            };
                            let mut response = WS2POkMessageV1 {
                                currency: currency.to_string(),
                                pubkey: signator.public_key(),
                                challenge: self.challenge.to_string(),
                                signature: None,
                            };
                            response.signature = Some(response.sign(signator));
                            return WS2Pv1MsgPayload::ValidAckMessage(
                                unwrap!(serde_json::to_string(&response)),
                                self.state,
                            );
                        } else {
                            warn!("The signature of message ACK is invalid !")
                        }
                    }
                    "OK" => {
                        let mut message = WS2POkMessageV1::parse(msg, currency.to_string())
                            .expect("Failed to parsing OK Message !");
                        trace!("Received OK");
                        message.challenge = self.remote_challenge.to_string();
                        message.pubkey = self.remote_pubkey.expect("fail to get remote pubkey !");
                        if message.verify() {
                            trace!("OK sig is valid.");
                            match self.state {
                                WS2PConnectionState::ConnectMessOk => {
                                    self.state = WS2PConnectionState::OkMessOkWaitingAckMess;
                                    return WS2Pv1MsgPayload::ValidOk(self.state);
                                }
                                WS2PConnectionState::AckMessOk => {
                                    info!(
                                        "WS2P Connection established with the key {}",
                                        self.remote_pubkey.expect("fail to get remote pubkey !")
                                    );
                                    self.state = WS2PConnectionState::Established;
                                    return WS2Pv1MsgPayload::ValidOk(self.state);
                                }
                                _ => {
                                    warn!("WS2P Error : OK message not expected !");
                                    return WS2Pv1MsgPayload::InvalidMessage;
                                }
                            }
                        } else {
                            warn!("The signature of message OK is invalid !");
                            return WS2Pv1MsgPayload::InvalidMessage;
                        }
                    }
                    &_ => debug!("unknow message"),
                };
            }
        };
        if let Some(req_id) = msg.get("reqId") {
            match req_id.as_str() {
                Some(req_id) => match msg.get("body") {
                    Some(body) => {
                        trace!("WS2P : Receive DB Request from {}.", self.node_full_id());

                        let req_id = match WS2Pv1ReqId::from_str(req_id) {
                            Ok(req_id) => req_id,
                            Err(_) => {
                                warn!(
                                    "WS2Pv1: receive invalid request: invalid req_id: '{}'",
                                    req_id
                                );
                                return WS2Pv1MsgPayload::WrongFormatMessage;
                            }
                        };

                        match WS2Pv1ReqBody::try_from(body) {
                            Ok(body) => {
                                return WS2Pv1MsgPayload::Request { req_id, body };
                            }
                            Err(_) => {
                                return WS2Pv1MsgPayload::WrongFormatMessage;
                            }
                        }
                    }
                    None => {
                        warn!("WS2P Error : invalid format : Request must contain a field body !");
                        return WS2Pv1MsgPayload::WrongFormatMessage;
                    }
                },
                None => {
                    warn!("WS2P Error : invalid format : Request must contain a field body !");
                    return WS2Pv1MsgPayload::WrongFormatMessage;
                }
            }
        }
        if let Some(req_id) = msg.get("resId") {
            match req_id.as_str() {
                Some(req_id_str) => match msg.get("body") {
                    Some(body) => match WS2Pv1ReqId::from_str(req_id_str) {
                        Ok(req_id) => {
                            return WS2Pv1MsgPayload::ReqResponse(req_id, body.clone());
                        }
                        Err(_) => {
                            return WS2Pv1MsgPayload::WrongFormatMessage;
                        }
                    },
                    None => match msg.get("err") {
                        Some(err) => warn!("Error in req : {:?}", err),
                        None => {
                            return WS2Pv1MsgPayload::WrongFormatMessage;
                        }
                    },
                },
                None => {
                    return WS2Pv1MsgPayload::WrongFormatMessage;
                }
            }
        }
        if let Some(body) = msg.get("body") {
            match body.get("name") {
                Some(s) => {
                    if s.is_string() {
                        match s.as_str().unwrap_or("") {
                            "BLOCK" => match body.get("block") {
                                Some(block) => match parse_json_block_from_serde_value(&block) {
                                    Ok(block_doc) => {
                                        return WS2Pv1MsgPayload::Document(DocumentDUBP::Block(
                                            Box::new(block_doc),
                                        ))
                                    }
                                    Err(e) => info!("WS2Pv1Signal: receive invalid block: {}", e),
                                },
                                None => return WS2Pv1MsgPayload::WrongFormatMessage,
                            },
                            "HEAD" => match body.get("heads") {
                                Some(heads) => match heads.as_array() {
                                    Some(heads_array) => {
                                        return WS2Pv1MsgPayload::Heads(heads_array.clone());
                                    }
                                    None => return WS2Pv1MsgPayload::WrongFormatMessage,
                                },
                                None => return WS2Pv1MsgPayload::WrongFormatMessage,
                            },
                            "PEER" => return self.parse_and_check_peer_message(body),
                            "CERTIFICATION" => {
                                trace!(
                                    "WS2P : Receive CERTIFICATION from {}.",
                                    self.node_full_id()
                                );
                                /*return WS2Pv1MsgPayload::Document(
                                    BlockchainDocument::Certification(_)
                                );*/
                            }
                            "IDENTITY" => {
                                trace!("WS2P : Receive IDENTITY from {}.", self.node_full_id());
                                /*return WS2Pv1MsgPayload::Document(
                                    BlockchainDocument::Identity(_)
                                );*/
                            }
                            "MEMBERSHIP" => {
                                trace!("WS2P : Receive MEMBERSHIP from {}.", self.node_full_id());
                                /*return WS2Pv1MsgPayload::Document(
                                    BlockchainDocument::Membership(_)
                                );*/
                            }
                            "TRANSACTION" => {
                                trace!("WS2P : Receive TRANSACTION from {}.", self.node_full_id());
                                /*return WS2Pv1MsgPayload::Document(
                                    BlockchainDocument::Transaction(_)
                                );*/
                            }
                            name => {
                                warn!(
                                    "WS2P : Receive unknown document name '{}' from '{}'.",
                                    name,
                                    self.node_full_id()
                                );
                                return WS2Pv1MsgPayload::UnknowMessage;
                            }
                        };
                    }
                }
                None => {
                    warn!("WS2P Error : invalid format : Body must contain a field name !");
                    return WS2Pv1MsgPayload::WrongFormatMessage;
                }
            }
        };
        debug!(
            "WS2P : Receive unknown message from '{}'.",
            self.node_full_id()
        );
        WS2Pv1MsgPayload::UnknowMessage
    }

    pub fn parse_and_check_peer_message(&mut self, body: &serde_json::Value) -> WS2Pv1MsgPayload {
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
                                                if ep.api == ApiName(String::from("WS2P")) {
                                                    ws2p_endpoints.push(ep);
                                                }
                                            }
                                        }
                                        WS2Pv1MsgPayload::PeerCard(body.clone(), ws2p_endpoints)
                                    }
                                    None => WS2Pv1MsgPayload::WrongFormatMessage,
                                },
                                None => WS2Pv1MsgPayload::WrongFormatMessage,
                            }
                        }
                        Err(_) => WS2Pv1MsgPayload::WrongFormatMessage,
                    }
                }
                None => WS2Pv1MsgPayload::WrongFormatMessage,
            },
            None => WS2Pv1MsgPayload::WrongFormatMessage,
        }
    }
}

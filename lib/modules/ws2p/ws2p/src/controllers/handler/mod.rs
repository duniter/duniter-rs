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

//! WS2P connection handler.

pub mod ack_msg;
pub mod connect_msg;
pub mod ok_msg;

use crate::constants::*;
use crate::controllers::*;
use crate::services::Ws2pServiceSender;
use crate::services::*;
use dubp_documents::CurrencyName;
use durs_common_tools::fatal_error;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::connect::generate_connect_message;
use durs_ws2p_messages::v2::payload_container::WS2Pv2MessagePayload;
use durs_ws2p_messages::v2::WS2Pv2Message;
use durs_ws2p_messages::WS2PMessage;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use ws::{util::Token, CloseCode, Handler, Handshake, Message};

const CONNECT: Token = Token(1);
const EXPIRE: Token = Token(2);
const RECV_SERVICE: Token = Token(3);

/// Our Handler struct.
/// Here we explicity indicate that the Ws2pConnectionHandler needs a Sender,
/// whereas a closure captures the Sender for us automatically.
#[derive(Debug)]
pub struct Ws2pConnectionHandler {
    /// Controller receiver
    pub receiver: mpsc::Receiver<Ws2pControllerOrder>,
    /// Websocket sender
    pub ws: WsSender,
    /// Service Sender
    pub service_sender: mpsc::Sender<Ws2pServiceSender>,
    /// Currency name
    pub currency: CurrencyName,
    /// Local node properties
    pub local_node: MySelfWs2pNode,
    /// Connection meta datas
    pub conn_datas: Ws2pConnectionDatas,
    /// Count invalid messages
    pub count_invalid_msgs: usize,
}

impl Ws2pConnectionHandler {
    /// Instantiate new Ws2pConnectionHandler
    pub fn try_new(
        ws: WsSender,
        service_sender: mpsc::Sender<Ws2pServiceSender>,
        currency: CurrencyName,
        local_node: MySelfWs2pNode,
        conn_datas: Ws2pConnectionDatas,
    ) -> Result<Ws2pConnectionHandler, mpsc::SendError<Ws2pServiceSender>> {
        // Create controller channel
        let (sender, receiver): (
            mpsc::Sender<Ws2pControllerOrder>,
            mpsc::Receiver<Ws2pControllerOrder>,
        ) = mpsc::channel();

        // Send controller sender to service
        debug!("Send controller sender to service");

        service_sender.send(Ws2pServiceSender::ControllerSender(sender))?;

        Ok(Ws2pConnectionHandler {
            receiver,
            ws,
            service_sender,
            currency,
            local_node,
            conn_datas,
            count_invalid_msgs: 0,
        })
    }
    fn send_new_conn_state_to_service(&self) {
        let remote_full_id = if let Some(remote_full_id) = self.conn_datas.remote_full_id {
            remote_full_id
        } else {
            NodeFullId::default()
        };
        self.service_sender
            .send(Ws2pServiceSender::ChangeConnectionState(
                remote_full_id,
                self.conn_datas.state,
            ))
            .expect("WS2p Service unreacheable !");
    }
    #[inline]
    fn update_status(&mut self, status: WS2PConnectionState) {
        self.conn_datas.state = status;
        self.send_new_conn_state_to_service();
    }
}

fn print_opt_addr(addr: Option<SocketAddr>) -> String {
    match addr {
        Some(addr) => format!("{}", addr),
        None => String::from(""),
    }
}

// We implement the Handler trait for Ws2pConnectionHandler so that we can get more
// fine-grained control of the connection.
impl Handler for Ws2pConnectionHandler {
    // `on_open` will be called only after the WebSocket handshake is successful
    // so at this point we know that the connection is ready to send/receive messages.
    // We ignore the `Handshake` for now, but you could also use this method to setup
    // Handler state or reject the connection based on the details of the Request
    // or Response, such as by checking cookies or Auth headers.
    fn on_open(&mut self, handshake: Handshake) -> ws::Result<()> {
        debug!(
            "open websocket from {}",
            print_opt_addr(handshake.peer_addr)
        );

        // Update connection state
        self.conn_datas.state = WS2PConnectionState::TryToSendConnectMsg;
        self.send_new_conn_state_to_service();

        // Generate connect message
        let connect_msg = generate_connect_message(
            self.conn_datas.connect_type,
            self.local_node.my_features.clone(),
            self.conn_datas.challenge,
            None,
        );

        // Encapsulate and binarize connect message
        if let Ok((_ws2p_full_msg, bin_connect_msg)) = WS2Pv2Message::encapsulate_payload(
            self.currency.clone(),
            self.local_node.my_node_id,
            self.local_node.my_key_pair,
            WS2Pv2MessagePayload::Connect(Box::new(connect_msg)),
        ) {
            // Start negociation timeouts
            self.ws.0.timeout(*WS2P_NEGOTIATION_TIMEOUT, CONNECT)?;
            // Start expire timeout
            self.ws
                .0
                .timeout(*WS2P_EXPIRE_TIMEOUT_IN_SECS * 1_000, EXPIRE)?;

            // Send connect message
            match self.ws.0.send(Message::binary(bin_connect_msg)) {
                Ok(()) => {
                    // Update state
                    if let WS2PConnectionState::TryToSendConnectMsg = self.conn_datas.state {
                        self.conn_datas.state = WS2PConnectionState::WaitingConnectMsg;
                        self.send_new_conn_state_to_service();
                    }
                    // Log
                    info!(
                        "Send CONNECT message to {}",
                        print_opt_addr(handshake.peer_addr)
                    );
                    debug!(
                        "Succesfully send CONNECT message to {}",
                        print_opt_addr(handshake.peer_addr)
                    );
                }
                Err(e) => {
                    self.conn_datas.state = WS2PConnectionState::Unreachable;
                    warn!(
                        "Fail to send CONNECT message to {} : {}",
                        print_opt_addr(handshake.peer_addr),
                        e
                    );
                    debug!(
                        "Fail send CONNECT message to {}",
                        print_opt_addr(handshake.peer_addr)
                    );
                    let _ = self
                        .ws
                        .0
                        .close_with_reason(CloseCode::Error, "Fail to send CONNECT message !");
                }
            }
            Ok(())
        } else {
            fatal_error!("Dev error: Fail to sign own connect message !");
        }
    }

    // `on_message` is roughly equivalent to the Handler closure. It takes a `Message`
    // and returns a `Result<()>`.
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        // Update last_mess_time
        self.conn_datas.last_mess_time = SystemTime::now();
        // Spam ?
        if SystemTime::now()
            .duration_since(self.conn_datas.last_mess_time)
            .unwrap()
            > Duration::new(*WS2P_SPAM_INTERVAL_IN_MILLI_SECS, 0)
        {
            if self.conn_datas.spam_interval {
                self.conn_datas.spam_counter += 1;
            } else {
                self.conn_datas.spam_interval = true;
                self.conn_datas.spam_counter = 2;
            }
        } else {
            self.conn_datas.spam_interval = false;
            self.conn_datas.spam_counter = 0;
        }
        // Spam ?
        if self.conn_datas.spam_counter >= *WS2P_SPAM_LIMIT {
            thread::sleep(Duration::from_millis(*WS2P_SPAM_SLEEP_TIME_IN_SEC));
            self.conn_datas.last_mess_time = SystemTime::now();
            return Ok(());
        }
        self.conn_datas.last_mess_time = SystemTime::now();

        if msg.is_binary() {
            debug!("Receive new message there is not a spam !");
            match WS2PMessage::parse_and_check_bin_message(&msg.into_data()) {
                Ok(valid_msg) => match valid_msg {
                    WS2PMessage::V2(msg_v2) => {
                        match msg_v2.payload {
                            WS2Pv2MessagePayload::Connect(ref box_connect_msg) => {
                                let connect_msg = box_connect_msg.deref();
                                // Get remote node id
                                let remote_full_id =
                                    NodeFullId(msg_v2.issuer_node_id, msg_v2.issuer_pubkey);
                                // Process connect message
                                connect_msg::process_ws2p_v2_connect_msg(
                                    self,
                                    remote_full_id,
                                    connect_msg,
                                );
                            }
                            WS2Pv2MessagePayload::Ack {
                                challenge: ack_msg_challenge,
                            } => {
                                // Process ack message
                                ack_msg::process_ws2p_v2_ack_msg(self, ack_msg_challenge);
                            }
                            WS2Pv2MessagePayload::SecretFlags(_) => {}
                            WS2Pv2MessagePayload::Ok(_) => {
                                // Process ok message
                                ok_msg::process_ws2p_v2_ok_msg(self);
                            }
                            WS2Pv2MessagePayload::Ko(_) => {}
                            _ => {
                                if let WS2PConnectionState::Established = self.conn_datas.state {

                                } else {
                                    let _ = self.ws.0.close_with_reason(
                                        CloseCode::Invalid,
                                        "Receive payload on negociation !",
                                    );
                                }
                            }
                        }
                    }
                },
                Err(ws2p_msg_err) => {
                    warn!("Message is invalid : {:?}", ws2p_msg_err);
                    self.count_invalid_msgs += 1;
                    if self.count_invalid_msgs >= *WS2P_INVALID_MSGS_LIMIT {
                        let _ = self.ws.0.close_with_reason(
                            CloseCode::Invalid,
                            "Receive several invalid messages !",
                        );
                    }
                }
            }
        } else if msg.is_text() {
            // ..
        }
        Ok(())
    }
    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        match event {
            CONNECT => {
                if let WS2PConnectionState::Established = self.conn_datas.state {
                    Ok(())
                } else {
                    self.conn_datas.state = WS2PConnectionState::NegociationTimeout;
                    self.ws
                        .0
                        .close_with_reason(CloseCode::Away, "negociation timeout")
                }
            }
            EXPIRE => {
                if SystemTime::now()
                    .duration_since(self.conn_datas.last_mess_time)
                    .expect("Sytem error")
                    .as_secs()
                    >= *WS2P_EXPIRE_TIMEOUT_IN_SECS
                {
                    self.conn_datas.state = WS2PConnectionState::Close;
                    self.ws
                        .0
                        .close_with_reason(CloseCode::Away, "Expire timeout")
                } else {
                    // Restart expire timeout
                    self.ws
                        .0
                        .timeout(*WS2P_EXPIRE_TIMEOUT_IN_SECS * 1_000, EXPIRE)?;
                    Ok(())
                }
            }
            RECV_SERVICE => {
                // Restart service timeout
                self.ws
                    .0
                    .timeout(*WS2P_RECV_SERVICE_FREQ_IN_MS, RECV_SERVICE)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    /*fn on_frame(&mut self, frame: Frame) -> ws::Result<Option<Frame>> {
        Ok(Some(frame))
    }*/
    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        /*match code {
            CloseCode::Normal => info!("The remote server close the connection."),
            CloseCode::Away => info!("The remote server is leaving."),
            _ => warn!("The remote server encountered an error: {}", reason),
        }
        let _result = self
            .conductor_sender
            .send(WS2PThreadSignal::WS2PConnectionMessage(
                WS2PConnectionMessage(
                    self.conn_datas.node_full_id(),
                    WS2PConnectionMessagePayload::Close,
                ),
            ));*/
    }
}

//  Copyright (C) 2018  The Dunitrust Project Developers.
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

use crate::constants;
use crate::controllers::WsSender;
use durs_common_tools::fatal_error;
use durs_message::DursMsg;
use durs_ws2p_protocol::connection_state::WS2PConnectionState;
use durs_ws2p_protocol::controller::{WS2PController, WebsocketActionOrder};
use durs_ws2p_protocol::websocket::{WebsocketAction, WebsocketIncomingEvent, WebsocketMessage};
use std::net::SocketAddr;

use ws::{util::Token, CloseCode, Handler, Handshake, Message};

const RECV_SERVICE: Token = Token(3);

/// Our Handler struct.
/// Here we explicity indicate that the Ws2pConnectionHandler needs a Sender,
/// whereas a closure captures the Sender for us automatically.
#[derive(Debug)]
pub struct Ws2pConnectionHandler {
    /// Websocket sender
    pub ws: WsSender,
    /// Remote addr
    pub remote_addr_opt: Option<SocketAddr>,
    /// WS2P Controller
    pub controller: WS2PController<DursMsg>,
}

impl Ws2pConnectionHandler {
    fn exec_ws_action(&mut self, ws_action_order: WebsocketActionOrder) -> ws::Result<()> {
        match ws_action_order.ws_action {
            WebsocketAction::ConnectTo { .. } => {
                fatal_error!("Could not generate a new connection in the context of a controller.")
            }
            WebsocketAction::SendMessage { msg } => {
                let ws_msg = match msg {
                    WebsocketMessage::Bin(bin_msg) => Message::binary(bin_msg),
                    WebsocketMessage::Str(str_msg) => Message::text(str_msg),
                };
                match self.ws.0.send(ws_msg) {
                    Ok(()) => {
                        // Update state
                        if let Some(new_state) = ws_action_order.new_state_if_success {
                            if let Err(e) = self.controller.update_conn_state(new_state) {
                                self.ws
                                    .0
                                    .close_with_reason(CloseCode::Error, format!("{}", e))?;
                            }
                        }
                        // Log
                        debug!(
                            "Succesfully send message to '{}'",
                            if let Some(remote_addr) = self.remote_addr_opt {
                                remote_addr.to_string()
                            } else {
                                "unknown addr".to_string()
                            }
                        );
                        Ok(())
                    }
                    Err(e) => {
                        let _ = self
                            .controller
                            .update_conn_state(ws_action_order.new_state_if_fail);
                        warn!(
                            "Fail to send message to '{}' : {}",
                            if let Some(remote_addr) = self.remote_addr_opt {
                                remote_addr.to_string()
                            } else {
                                "unknown addr".to_string()
                            },
                            e
                        );
                        self.ws
                            .0
                            .close_with_reason(CloseCode::Error, "Fail to send message !")
                    }
                }
            }
            WebsocketAction::CloseConnection { reason } => match self.ws.0.close_with_reason(
                CloseCode::Error,
                reason.unwrap_or_else(|| String::from("unknown reason")),
            ) {
                Ok(()) => {
                    if let Some(new_state) = ws_action_order.new_state_if_success {
                        let _ = self.controller.update_conn_state(new_state);
                    }
                    Ok(())
                }
                Err(e) => {
                    let _ = self
                        .controller
                        .update_conn_state(ws_action_order.new_state_if_fail);
                    Err(e)
                }
            },
        }
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
        self.remote_addr_opt = handshake.peer_addr;
        match self.controller.process(WebsocketIncomingEvent::OnOpen {
            remote_addr: self.remote_addr_opt,
        }) {
            Ok(ws_action_order_opt) => {
                // Start RECV_SERVICE timeout
                self.ws
                    .0
                    .timeout(*constants::WS2P_RECV_SERVICE_FREQ_IN_MS, RECV_SERVICE)?;
                // Execute websocket action order
                if let Some(ws_action_order) = ws_action_order_opt {
                    self.exec_ws_action(ws_action_order)
                } else {
                    Ok(())
                }
            }
            Err(e) => self.exec_ws_action(WebsocketActionOrder {
                ws_action: WebsocketAction::CloseConnection {
                    reason: Some(format!("{}", e)),
                },
                new_state_if_success: Some(WS2PConnectionState::Close),
                new_state_if_fail: WS2PConnectionState::Unreachable,
            }),
        }
    }

    // `on_message` is roughly equivalent to the Handler closure. It takes a `Message`
    // and returns a `Result<()>`.
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        let msg = match msg {
            Message::Binary(bin_msg) => WebsocketMessage::Bin(bin_msg),
            Message::Text(str_msg) => WebsocketMessage::Str(str_msg),
        };
        match self
            .controller
            .process(WebsocketIncomingEvent::OnMessage { msg })
        {
            Ok(ws_action_order_opt) => {
                if let Some(ws_action_order) = ws_action_order_opt {
                    self.exec_ws_action(ws_action_order)
                } else {
                    Ok(())
                }
            }
            Err(e) => self.exec_ws_action(WebsocketActionOrder {
                ws_action: WebsocketAction::CloseConnection {
                    reason: Some(format!("{}", e)),
                },
                new_state_if_success: Some(WS2PConnectionState::Close),
                new_state_if_fail: WS2PConnectionState::Unreachable,
            }),
        }
    }
    fn on_timeout(&mut self, _event: Token) -> ws::Result<()> {
        self.ws.0.timeout(1_000, RECV_SERVICE)?;
        if let Some(ws_action_order) = self.controller.check_timeouts() {
            self.exec_ws_action(ws_action_order)
        } else {
            Ok(())
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

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

//! WS2P connections handler.

use super::messages::*;
use super::meta_datas::WS2PConnectionMetaDatas;
use super::states::WS2PConnectionState;
use crate::constants::*;
use crate::*;
use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use log::error;
use std::sync::mpsc;
#[allow(deprecated)]
use ws::util::{Timeout, Token};
use ws::{CloseCode, Frame, Handler, Handshake, Message, Sender};

const CONNECT: Token = Token(1);
const EXPIRE: Token = Token(2);

// Our Handler struct.
// Here we explicity indicate that the Client needs a Sender,
// whereas a closure captures the Sender for us automatically.
#[allow(deprecated)]
#[derive(Debug)]
pub struct Client {
    ws: Sender,
    conductor_sender: mpsc::Sender<WS2PThreadSignal>,
    currency: String,
    connect_message: Message,
    conn_meta_datas: WS2PConnectionMetaDatas,
    last_mess_time: SystemTime,
    signator: SignatorEnum,
    spam_interval: bool,
    spam_counter: usize,
    timeout: Option<Timeout>,
}

pub fn connect_to_ws2p_endpoint(
    endpoint: &EndpointV1,
    conductor_sender: &mpsc::Sender<WS2PThreadSignal>,
    currency: &str,
    keypair: &KeyPairEnum,
) -> ws::Result<()> {
    // Get endpoint url
    let ws_url = endpoint.get_url(true, false).expect("Endpoint unreachable");

    // Create WS2PConnectionMetaDatass
    let mut conn_meta_datas = WS2PConnectionMetaDatas::new(
        "b60a14fd-0826-4ae0-83eb-1a92cd59fd5308535fd3-78f2-4678-9315-cd6e3b7871b1".to_string(),
    );
    conn_meta_datas.remote_pubkey = Some(endpoint.issuer);
    conn_meta_datas.remote_uuid = Some(
        endpoint
            .node_id
            .expect("WS2P: Fail to get ep.node_uuid() !"),
    );

    // Log
    info!("WS2P: Try connection to {} ...", ws_url);

    // Connect to websocket
    ws::connect(ws_url, |ws| {
        // Generate signator
        let signator = if let Ok(signator) = keypair.generate_signator() {
            signator
        } else {
            fatal_error!("Your key pair is corrupted, please recreate it !");
        };

        // Generate connect message
        let connect_message =
            generate_connect_message(currency, &signator, conn_meta_datas.challenge.clone());

        Client {
            ws,
            conductor_sender: conductor_sender.clone(),
            currency: String::from(currency),
            connect_message: connect_message.clone(),
            conn_meta_datas: conn_meta_datas.clone(),
            last_mess_time: SystemTime::now(),
            signator,
            spam_interval: false,
            spam_counter: 0,
            timeout: None,
        }
    })
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
            .send(WS2PThreadSignal::WS2Pv1Msg(WS2Pv1Msg {
                from: self.conn_meta_datas.node_full_id(),
                payload: WS2Pv1MsgPayload::WebsocketOk(WsSender(self.ws.clone())),
            }));
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
        if unwrap!(SystemTime::now().duration_since(self.last_mess_time))
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
            debug!("WS2P: receive mess: {}", s);
            let json_message: serde_json::Value = serde_json::from_str(&s)
                .expect("WS2P: Fail to convert string message ton json value !");
            let result = self
                .conductor_sender
                .send(WS2PThreadSignal::WS2Pv1Msg(WS2Pv1Msg {
                    from: self.conn_meta_datas.node_full_id(),
                    payload: self.conn_meta_datas.parse_and_check_incoming_message(
                        &self.currency,
                        &self.signator,
                        &json_message,
                    ),
                }));
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
                            .send(WS2PThreadSignal::WS2Pv1Msg(WS2Pv1Msg {
                                from: self.conn_meta_datas.node_full_id(),
                                payload: WS2Pv1MsgPayload::NegociationTimeout,
                            }));
                    self.ws.close(CloseCode::Away)
                } else {
                    Ok(())
                }
            }
            EXPIRE => {
                let _result = self
                    .conductor_sender
                    .send(WS2PThreadSignal::WS2Pv1Msg(WS2Pv1Msg {
                        from: self.conn_meta_datas.node_full_id(),
                        payload: WS2Pv1MsgPayload::Timeout,
                    }));
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
            .send(WS2PThreadSignal::WS2Pv1Msg(WS2Pv1Msg {
                from: self.conn_meta_datas.node_full_id(),
                payload: WS2Pv1MsgPayload::Close,
            }));
    }
}

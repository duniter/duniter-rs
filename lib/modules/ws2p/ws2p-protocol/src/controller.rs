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

//! WebSocketToPeer V2+ API Protocol.
//! Controller manage one WS2P connection.

pub mod meta_datas;
mod on_message;
mod on_open;

use self::meta_datas::WS2PControllerMetaDatas;
use crate::connection_state::WS2PConnectionState;
use crate::constants;
use crate::orchestrator::OrchestratorMsg;
use crate::websocket::{WebsocketAction, WebsocketIncomingEvent};
use durs_module::ModuleMessage;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use durs_ws2p_messages::WS2PMessage;
use failure::Fail;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::time::Instant;

#[derive(Copy, Clone, Debug, Hash)]
/// WS2P Controller unique identitier
pub enum WS2PControllerId {
    /// Client controller
    Client {
        /// Expected remote node full id
        expected_remote_full_id: Option<NodeFullId>,
    },
    /// Server Incoming controller
    Incoming,
    /// Server outgoing controller
    Outgoing {
        /// Expected remote node full id
        expected_remote_full_id: Option<NodeFullId>,
    },
}

impl WS2PControllerId {
    /// Get expected remote node full id
    pub fn expected_remote_full_id(&self) -> Option<NodeFullId> {
        match self {
            WS2PControllerId::Client {
                expected_remote_full_id,
            }
            | WS2PControllerId::Outgoing {
                expected_remote_full_id,
            } => *expected_remote_full_id,
            WS2PControllerId::Incoming => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Event transmitted to the orchestrator
pub enum WS2PControllerEvent {
    /// New connection established
    NewConnEstablished {
        /// Connection type
        conn_type: WS2Pv2ConnectType,
        /// Remote node full id
        remote_full_id: NodeFullId,
    },
    /// Connection state change
    StateChange {
        /// New connection state
        new_state: WS2PConnectionState,
    },
    /// The controller only reports a message if it cannot process it entirely on its own.
    /// For example, connection negotiation messages are not sent back.
    RecvValidMsg {
        /// WS2P Message
        ws2p_msg: WS2PMessage,
    },
}

#[derive(Debug)]
/// WS2P Controller
pub struct WS2PController<M: ModuleMessage> {
    /// Controller id
    pub id: WS2PControllerId,
    /// Orchestrator sender
    pub orchestrator_sender: Sender<OrchestratorMsg<M>>,
    /// Controller meta datas
    pub meta_datas: WS2PControllerMetaDatas,
    /// Controller receiver
    pub receiver: Receiver<WebsocketActionOrder>,
}

#[derive(Copy, Clone, Debug, Fail)]
/// WS2P Controller process error
pub enum WS2PControllerProcessError {
    /// Orchestrator unreacheable
    #[fail(display = "WS2P Orchestrator unreachable")]
    OrchestratorUnreacheable,
}

/// Websocket action order
#[derive(Clone, Debug)]
pub struct WebsocketActionOrder {
    /// Websocket actio,
    pub ws_action: WebsocketAction,
    /// New state if action success
    pub new_state_if_success: Option<WS2PConnectionState>,
    /// New state if action fail
    pub new_state_if_fail: WS2PConnectionState,
}

impl WebsocketActionOrder {
    /// Close connection
    #[inline]
    pub fn close() -> Self {
        WebsocketActionOrder::close_with_reason(None)
    }
    /// Close connection with reason
    #[inline]
    pub fn close_with_reason(reason: Option<String>) -> Self {
        WebsocketActionOrder {
            ws_action: WebsocketAction::CloseConnection { reason },
            new_state_if_success: Some(WS2PConnectionState::Close),
            new_state_if_fail: WS2PConnectionState::Unreachable,
        }
    }
}

impl<M: ModuleMessage> WS2PController<M> {
    /// Check timeouts
    pub fn check_timeouts(&mut self) -> Option<WebsocketActionOrder> {
        let now = Instant::now();

        if self.meta_datas.state == WS2PConnectionState::Established {
            if now.duration_since(self.meta_datas.last_mess_time).as_secs()
                > *constants::WS2P_EXPIRE_TIMEOUT_IN_SECS
            {
                Some(WebsocketActionOrder {
                    ws_action: WebsocketAction::CloseConnection {
                        reason: Some("Closing due to inactivity.".to_owned()),
                    },
                    new_state_if_success: Some(WS2PConnectionState::Close),
                    new_state_if_fail: WS2PConnectionState::Unreachable,
                })
            } else {
                None
            }
        } else if now.duration_since(self.meta_datas.creation_time).as_secs()
            > *constants::WS2P_NEGOTIATION_TIMEOUT_IN_SECS
        {
            Some(WebsocketActionOrder {
                ws_action: WebsocketAction::CloseConnection {
                    reason: Some("Negociation timeout.".to_owned()),
                },
                new_state_if_success: Some(WS2PConnectionState::Close),
                new_state_if_fail: WS2PConnectionState::Unreachable,
            })
        } else {
            None
        }
    }

    /// Try to instanciate new controller
    pub fn try_new(
        id: WS2PControllerId,
        meta_datas: WS2PControllerMetaDatas,
        orchestrator_sender: Sender<OrchestratorMsg<M>>,
    ) -> Result<WS2PController<M>, SendError<OrchestratorMsg<M>>> {
        let (sender, receiver) = std::sync::mpsc::channel();

        orchestrator_sender.send(OrchestratorMsg::ControllerSender(sender))?;

        Ok(WS2PController {
            id,
            meta_datas,
            orchestrator_sender,
            receiver,
        })
    }

    /// Get orchestrator sender
    pub fn get_pending_ws_actions(&self) -> Vec<WebsocketActionOrder> {
        let mut ws_actions = Vec::new();

        while let Ok(ws_action) = self.receiver.recv() {
            ws_actions.push(ws_action);
        }

        ws_actions
    }

    /// Process a websocket incoming event
    pub fn process(
        &mut self,
        event: WebsocketIncomingEvent,
    ) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
        match event {
            WebsocketIncomingEvent::OnOpen { remote_addr } => on_open::process(self, remote_addr),
            WebsocketIncomingEvent::OnMessage { msg } => on_message::process(self, msg),
            WebsocketIncomingEvent::OnClose { close_code, reason } => {
                let remote_str = if let Some(remote_node) = &self.meta_datas.remote_node {
                    remote_node.remote_full_id.to_string()
                } else {
                    "unknow".to_owned()
                };
                log::warn!(
                    "Connection with remote '{}' closed (close_code={}, reason={}).",
                    remote_str,
                    close_code,
                    reason.unwrap_or_else(|| "".to_owned())
                );
                self.update_conn_state(WS2PConnectionState::Close)?;
                Ok(None)
            }
        }
    }

    fn send_event(&mut self, event: WS2PControllerEvent) -> Result<(), WS2PControllerProcessError> {
        if self
            .orchestrator_sender
            .send(OrchestratorMsg::ControllerEvent {
                controller_id: self.id,
                event,
            })
            .is_err()
            && self.meta_datas.state != WS2PConnectionState::Close
        {
            Err(WS2PControllerProcessError::OrchestratorUnreacheable)
        } else {
            Ok(())
        }
    }

    #[inline]
    /// Update connection state
    pub fn update_conn_state(
        &mut self,
        new_state: WS2PConnectionState,
    ) -> Result<(), WS2PControllerProcessError> {
        self.meta_datas.state = new_state;
        self.send_event(WS2PControllerEvent::StateChange { new_state })
    }
}

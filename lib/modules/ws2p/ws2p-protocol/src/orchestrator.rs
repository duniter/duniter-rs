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
//! Orchestrator manage WS2P Node.

use std::sync::mpsc::Sender;

use crate::controller::{WS2PControllerEvent, WS2PControllerId, WebsocketActionOrder};
use durs_module::ModuleMessage;

/// Orchestrator message
#[derive(Debug)]
pub enum OrchestratorMsg<M: ModuleMessage> {
    /// Controller sender
    ControllerSender(Sender<WebsocketActionOrder>),
    /// Controller event
    ControllerEvent {
        /// Controller unique identifier
        controller_id: WS2PControllerId,
        /// Controller event
        event: WS2PControllerEvent,
    },
    /// Module message
    ModuleMessage(M),
}

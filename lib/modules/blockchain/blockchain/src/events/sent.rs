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

//! Sub-module managing the events emitted by the blockchain module.

use crate::*;
use durs_module::ModuleEvent;
use durs_message::events::BlockchainEvent;

/// Send blockchain event
pub fn send_event(bc: &BlockchainModule, event: &BlockchainEvent) {
    let module_event = match event {
        BlockchainEvent::StackUpValidBlock(_) => ModuleEvent::NewValidBlock,
        BlockchainEvent::RevertBlocks(_) => ModuleEvent::RevertBlocks,
        _ => return,
    };
    bc.router_sender
        .send(RouterThreadMessage::ModuleMessage(DursMsg::Event {
            event_type: module_event,
            event_content: DursEvent::BlockchainEvent(Box::new(event.clone())),
        }))
        .unwrap_or_else(|_| panic!("Fail to send BlockchainEvent to router"));
}

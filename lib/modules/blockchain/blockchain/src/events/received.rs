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

//! Sub-module managing events received from other durs modules

use crate::*;
use duniter_module::*;
use durs_message::events::DursEvent;
use std::ops::Deref;

pub fn receive_event(
    bc: &mut BlockchainModule,
    _event_type: ModuleEvent,
    event_content: &DursEvent,
) {
    match *event_content {
        DursEvent::NetworkEvent(ref network_event_box) => match *network_event_box.deref() {
            NetworkEvent::ReceiveDocuments(ref network_docs) => {
                dunp::receiver::receive_bc_documents(bc, network_docs);
            }
            NetworkEvent::ReceiveHeads(_) => {}
            _ => {}
        },
        DursEvent::MemPoolEvent(ref mempool_event) => {
            if let MemPoolEvent::FindNextBlock(next_block_box) = mempool_event {
                dunp::receiver::receive_blocks(
                    bc,
                    vec![Block::LocalBlock(next_block_box.deref().clone())],
                );
            }
        }
        _ => {} // Others modules events
    }
}

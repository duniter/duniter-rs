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
use durs_message::events::DursEvent;
use durs_module::*;
use std::ops::Deref;

pub fn receive_event(
    bc: &mut BlockchainModule,
    _event_type: ModuleEvent,
    event_content: DursEvent,
) {
    match event_content {
        DursEvent::NetworkEvent(network_event) => match network_event {
            NetworkEvent::ReceiveDocuments(network_docs) => {
                dunp::receiver::receive_user_documents(bc, &network_docs);
            }
            NetworkEvent::ReceiveBlocks(blocks) => {
                dunp::receiver::receive_blocks(bc, blocks);
            }
            NetworkEvent::ReceiveHeads(_) => {}
            _ => {}
        },
        DursEvent::MemPoolEvent(mempool_event) => {
            if let MemPoolEvent::FindNextBlock(next_block_box) = mempool_event {
                dunp::receiver::receive_blocks(bc, vec![next_block_box.deref().clone()]);
            }
        }
        _ => {} // Others modules events
    }
}

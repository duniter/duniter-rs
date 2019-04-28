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
use dubp_documents::Document;
use durs_message::events::DursEvent;
use durs_module::*;
use std::ops::Deref;

pub fn receive_event(
    ws2p_module: &mut WS2PModule,
    _event_type: ModuleEvent,
    event_content: &DursEvent,
) {
    if let DursEvent::BlockchainEvent(ref bc_event) = *event_content {
        match *bc_event.deref() {
            BlockchainEvent::StackUpValidBlock(ref block) => {
                ws2p_module.current_blockstamp = block.deref().blockstamp();
                debug!(
                    "WS2PModule : current_blockstamp = {}",
                    ws2p_module.current_blockstamp
                );
                ws2p_module.my_head = Some(heads::generate_my_head(
                    &ws2p_module.key_pair,
                    ws2p_module.node_id,
                    ws2p_module.soft_name,
                    ws2p_module.soft_version,
                    &ws2p_module.current_blockstamp,
                    None,
                ));
                super::sent::send_network_event(
                    ws2p_module,
                    NetworkEvent::ReceiveHeads(vec![unwrap!(ws2p_module.my_head.clone())]),
                );
                // Send my head to all connections
                let my_json_head = serializer::serialize_head(unwrap!(ws2p_module.my_head.clone()));
                trace!("Send my HEAD: {:#?}", my_json_head);
                let _results: Result<(), ws::Error> = ws2p_module
                    .websockets
                    .iter_mut()
                    .map(|ws| {
                        (ws.1).0.send(Message::text(
                            json!({
                                "name": "HEAD",
                                "body": {
                                    "heads": [my_json_head]
                                }
                            })
                            .to_string(),
                        ))
                    })
                    .collect();
            }
            BlockchainEvent::RevertBlocks(ref _blocks) => {}
            _ => {}
        }
    }
}

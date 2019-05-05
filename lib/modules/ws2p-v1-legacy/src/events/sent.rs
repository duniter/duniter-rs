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

use crate::WS2PModule;
use durs_message::events::DursEvent;
use durs_message::*;
use durs_module::{ModuleEvent, RouterThreadMessage};
use durs_network::documents::BlockchainDocument;
use durs_network::events::NetworkEvent;

pub fn send_network_events(ws2p_module: &mut WS2PModule, events: Vec<NetworkEvent>) {
    for event in events {
        send_network_event(ws2p_module, event);
    }
}

pub fn send_network_event(ws2p_module: &mut WS2PModule, event: NetworkEvent) {
    let module_event = match event {
        NetworkEvent::ConnectionStateChange(_, _, _, _) => {
            ModuleEvent::ConnectionsChangeNodeNetwork
        }
        NetworkEvent::ReceiveBlocks(_) => ModuleEvent::NewBlockFromNetwork,
        NetworkEvent::ReceiveDocuments(ref network_docs) => {
            if !network_docs.is_empty() {
                match network_docs[0] {
                    BlockchainDocument::Block(_) => ModuleEvent::NewBlockFromNetwork,
                    BlockchainDocument::Transaction(_) => ModuleEvent::NewTxFromNetwork,
                    _ => ModuleEvent::NewWotDocFromNetwork,
                }
            } else {
                return;
            }
        }
        NetworkEvent::ReceiveHeads(_) => ModuleEvent::NewValidHeadFromNetwork,
        NetworkEvent::ReceivePeers(_) => ModuleEvent::NewValidPeerFromNodeNetwork,
        NetworkEvent::NewSelfPeer(_) => ModuleEvent::NewSelfPeer,
    };
    ws2p_module
        .router_sender
        .send(RouterThreadMessage::ModuleMessage(DursMsg::Event {
            event_type: module_event,
            event_content: DursEvent::NetworkEvent(event),
        }))
        .expect("Fail to send network event to router !");
}

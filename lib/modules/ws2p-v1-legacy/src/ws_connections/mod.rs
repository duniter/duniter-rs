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

//! Manage websockets connections.

pub mod handler;
pub mod messages;
mod meta_datas;
pub mod requests;
pub mod responses;
pub mod states;

use crate::*;
use dup_crypto::keys::*;
use durs_network::documents::BlockchainDocument;
use durs_network_documents::network_endpoint::EndpointV1;
use rand::Rng;
use states::WS2PConnectionState;
use std::cmp::Ordering;
use std::collections::HashSet;
#[allow(deprecated)]
use ws::Sender;

/// Store a websocket sender
pub struct WsSender(pub Sender);

impl ::std::fmt::Debug for WsSender {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "WsSender {{ }}")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WS2PCloseConnectionReason {
    AuthMessInvalidSig,
    NegociationTimeout,
    Timeout,
    WsError,
    Unknow,
}

pub fn connect_to_know_endpoints(ws2p_module: &mut WS2Pv1Module) {
    info!("WS2P: connect to know endpoints...");
    let mut count_established_connections = 0;
    let mut pubkeys = HashSet::new();
    let mut reachable_endpoints = Vec::new();
    let mut unreachable_endpoints = Vec::new();
    for (_ws2p_full_id, DbEndpoint { ep, state, .. }) in ws2p_module.ws2p_endpoints.clone() {
        if ep.issuer == ws2p_module.key_pair.public_key() || !pubkeys.contains(&ep.issuer) {
            match state {
                WS2PConnectionState::Established => count_established_connections += 1,
                WS2PConnectionState::NeverTry
                | WS2PConnectionState::Close
                | WS2PConnectionState::Denial => {
                    pubkeys.insert(ep.issuer);
                    if ws2p_module.ssl || ep.port != 443 {
                        reachable_endpoints.push(ep);
                    }
                }
                _ => {
                    pubkeys.insert(ep.issuer);
                    unreachable_endpoints.push(ep);
                }
            }
        }
    }
    if !ws2p_module.conf.prefered_pubkeys.is_empty() {
        reachable_endpoints.sort_unstable_by(|ep1, ep2| {
            if ws2p_module.conf.prefered_pubkeys.contains(&ep1.issuer) {
                if ws2p_module.conf.prefered_pubkeys.contains(&ep2.issuer) {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        });
    }
    let mut free_outcoming_rooms =
        ws2p_module.conf.clone().outcoming_quota - count_established_connections;
    while free_outcoming_rooms > 0 {
        let ep = if !reachable_endpoints.is_empty() {
            reachable_endpoints
                .pop()
                .expect("WS2P: Fail to pop() reachable_endpoints !")
        } else if !unreachable_endpoints.is_empty() {
            unreachable_endpoints
                .pop()
                .expect("WS2P: Fail to pop() unreachable_endpoints !")
        } else {
            break;
        };
        connect_to_without_checking_quotas(ws2p_module, unwrap!(ep.node_full_id()));
        free_outcoming_rooms -= 1;
    }
}

pub fn connect_to(ws2p_module: &mut WS2Pv1Module, ep: &EndpointV1) {
    // Add endpoint to endpoints list (if there isn't already)
    let node_full_id = ep
        .node_full_id()
        .expect("WS2P: Fail to get ep.node_full_id() !");
    ws2p_module
        .ws2p_endpoints
        .entry(node_full_id)
        .or_insert(DbEndpoint {
            ep: ep.clone(),
            state: WS2PConnectionState::NeverTry,
            last_check: 0,
        });
    let count_established_connections = count_established_connections(&ws2p_module);
    if ws2p_module.conf.outcoming_quota > count_established_connections {
        connect_to_without_checking_quotas(ws2p_module, node_full_id);
    }
}

pub fn connect_to_without_checking_quotas(
    ws2p_module: &mut WS2Pv1Module,
    node_full_id: NodeFullId,
) {
    let endpoint = unwrap!(ws2p_module.ws2p_endpoints.get(&node_full_id));
    let endpoint_copy = endpoint.ep.clone();
    let conductor_sender_copy = ws2p_module.main_thread_channel.0.clone();
    let currency_copy = ws2p_module.conf.currency.clone();
    let key_pair_copy = ws2p_module.key_pair;
    thread::spawn(move || {
        let _result = crate::ws_connections::handler::connect_to_ws2p_endpoint(
            &endpoint_copy,
            &conductor_sender_copy,
            &currency_copy.expect("WS2PError : No currency !").0,
            key_pair_copy,
        );
    });
}

pub fn close_connection(
    ws2p_module: &mut WS2Pv1Module,
    ws2p_full_id: &NodeFullId,
    reason: WS2PCloseConnectionReason,
) {
    match reason {
        WS2PCloseConnectionReason::NegociationTimeout => {}
        WS2PCloseConnectionReason::AuthMessInvalidSig
        | WS2PCloseConnectionReason::Timeout
        | WS2PCloseConnectionReason::WsError
        | WS2PCloseConnectionReason::Unknow => {
            if let Some(dal_ep) = ws2p_module.ws2p_endpoints.get_mut(ws2p_full_id) {
                dal_ep.state = WS2PConnectionState::Close;
                dal_ep.last_check = durs_common_tools::fns::time::current_timestamp();
            }
        }
    }
    if let Some(websocket) = ws2p_module.websockets.get(&ws2p_full_id) {
        let _result = websocket.0.close(ws::CloseCode::Normal);
    }
    let _result = ws2p_module.websockets.remove(ws2p_full_id);
}

pub fn get_random_connection<S: ::std::hash::BuildHasher>(
    connections: HashSet<&NodeFullId, S>,
) -> NodeFullId {
    let mut rng = rand::thread_rng();
    let mut loop_count = 0;
    loop {
        for ws2p_full_id in &connections {
            if loop_count > 10 {
                return **ws2p_full_id;
            }
            if rng.gen::<bool>() {
                return **ws2p_full_id;
            }
        }
        loop_count += 1;
    }
}

pub fn count_established_connections(ws2p_module: &WS2Pv1Module) -> usize {
    let mut count_established_connections = 0;
    for DbEndpoint { state, .. } in ws2p_module.ws2p_endpoints.values() {
        if let WS2PConnectionState::Established = state {
            count_established_connections += 1;
        }
    }
    count_established_connections
}

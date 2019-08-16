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

//! WS2P outgoing Services

use crate::services::WsError;
use crate::*;
use dubp_currency_params::CurrencyName;
use durs_network_documents::{NodeFullId, NodeId};
use durs_ws2p_protocol::connection_state::WS2PConnectionState;
use durs_ws2p_protocol::controller::WebsocketActionOrder;
use durs_ws2p_protocol::orchestrator::OrchestratorMsg;
use durs_ws2p_protocol::MySelfWs2pNode;
use std::collections::HashMap;
use std::sync::mpsc;

#[derive(Debug, Clone)]
/// Data allowing the service to manage an outgoing connection
pub struct OutgoingConnection {
    /// Endpoint
    pub endpoint: EndpointEnum,
    /// Controller channel
    pub controller: mpsc::Sender<WebsocketActionOrder>,
}

#[derive(Debug, Copy, Clone)]
/// Endpoind whose last connection attempt failed
pub struct EndpointInError {
    /// Last attemp time
    pub last_attempt_time: u64,
    /// Error status
    pub error: WS2PConnectionState,
}

#[derive(Debug)]
/// Outgoing connection management service
pub struct WS2POutgoingOrchestrator {
    /// Currency Name
    pub currency: CurrencyName,
    /// Local node datas
    pub self_node: MySelfWs2pNode,
    /// Outgoing connections quota
    pub quota: usize,
    /// List of established connections
    pub connections: HashMap<NodeFullId, OutgoingConnection>,
    /// List of endpoinds whose last connection attempt failed
    pub endpoints_in_error: HashMap<NodeFullId, EndpointInError>,
    /// List of endpoints that have never been contacted
    pub never_try_endpoints: Vec<EndpointEnum>,
    /// Service receiver
    pub receiver: mpsc::Receiver<OrchestratorMsg<DursMsg>>,
    /// Orchestrator sender
    pub sender: mpsc::Sender<OrchestratorMsg<DursMsg>>,
}

impl WS2POutgoingOrchestrator {
    /// Instantiate WS2POutgoingOrchestrator
    pub fn new(
        currency: CurrencyName,
        ws2p_conf: &WS2PConf,
        self_node: MySelfWs2pNode,
    ) -> WS2POutgoingOrchestrator {
        // Create service channel
        let (sender, receiver) = mpsc::channel();

        WS2POutgoingOrchestrator {
            currency,
            quota: ws2p_conf.outcoming_quota,
            connections: HashMap::with_capacity(ws2p_conf.outcoming_quota),
            endpoints_in_error: HashMap::new(),
            never_try_endpoints: Vec::new(),
            self_node,
            receiver,
            sender,
        }
    }

    /// Connect to WSPv2 Endpoint
    pub fn connect_to_ws2p_v2_endpoint(
        &self,
        endpoint: &EndpointEnum,
        remote_node_id: Option<NodeId>,
    ) -> Result<(), WsError> {
        let expected_remote_full_id = if let Some(remote_node_id) = remote_node_id {
            Some(NodeFullId(remote_node_id, endpoint.pubkey()))
        } else {
            None
        };
        match controllers::outgoing_connections::connect_to_ws2p_v2_endpoint(
            &self.currency,
            &self.sender,
            &self.self_node,
            expected_remote_full_id,
            endpoint,
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(WsError::UnknownError),
        }
    }
}

//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Sub module define WS2P controller meta datas

use crate::connection_state::WS2PConnectionState;
use crate::MySelfWs2pNode;
use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use dup_currency_params::CurrencyName;
use durs_network_documents::network_peer::PeerCardV11;
use durs_network_documents::NodeFullId;
use durs_ws2p_messages::v2::api_features::WS2PFeatures;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use std::time::SystemTime;

#[derive(Debug, Clone)]
/// WS2p Connection meta datas
pub struct WS2PControllerMetaDatas {
    /// Local challenge
    pub challenge: Hash,
    /// connect type
    pub connect_type: WS2Pv2ConnectType,
    /// Count invalid messages
    pub count_invalid_msgs: usize,
    /// Currency name
    pub currency: CurrencyName,
    /// Controller creation time
    pub creation_time: SystemTime,
    /// Connection features
    pub features: Option<WS2PFeatures>,
    /// Timestamp of last received message
    pub last_mess_time: SystemTime,
    /// Local node properties
    pub local_node: MySelfWs2pNode,
    /// Remote connect type
    pub remote_connect_type: Option<WS2Pv2ConnectType>,
    /// Remote node datas
    pub remote_node: Option<Ws2pRemoteNodeDatas>,
    /// Indicator required for the anti-spam mechanism
    pub spam_interval: bool,
    /// Indicator required for the anti-spam mechanism
    pub spam_counter: usize,
    /// Connection state
    pub state: WS2PConnectionState,
}

impl WS2PControllerMetaDatas {
    /// Instanciate new WS2PControllerMetaDatas
    pub fn new(
        challenge: Hash,
        connect_type: WS2Pv2ConnectType,
        currency: CurrencyName,
        local_node: MySelfWs2pNode,
    ) -> Self {
        WS2PControllerMetaDatas {
            challenge,
            connect_type,
            count_invalid_msgs: 0,
            currency,
            creation_time: SystemTime::now(),
            features: None,
            last_mess_time: SystemTime::now(),
            local_node,
            remote_connect_type: None,
            remote_node: None,
            spam_interval: false,
            spam_counter: 0,
            state: WS2PConnectionState::TryToOpenWS,
        }
    }
}

#[derive(Debug, Clone)]
/// WS2P remote node datas
pub struct Ws2pRemoteNodeDatas {
    /// Remote challenge
    pub challenge: Hash,
    /// Remote current blockstamp
    pub current_blockstamp: Option<Blockstamp>,
    /// Remote peer card
    pub peer_card: Option<PeerCardV11>,
    /// Remote full id
    pub remote_full_id: NodeFullId,
}

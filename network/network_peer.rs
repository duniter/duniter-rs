//  Copyright (C) 2017  The Duniter Project Developers.
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

//! Module defining the format of network peer cards and how to handle them.

extern crate crypto;
extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_module;
extern crate serde;
extern crate serde_json;

use super::network_endpoint::NetworkEndpoint;
use duniter_crypto::keys::*;
use duniter_documents::Blockstamp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card V10
pub struct NetworkPeerV10 {
    /// Peer card Blockstamp
    pub blockstamp: Blockstamp,
    /// Peer card issuer
    pub issuer: PubKey,
    /// Peer card endpoints list
    pub endpoints: Vec<NetworkEndpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Peer card
pub enum NetworkPeer {
    /// Peer card V10
    V10(NetworkPeerV10),
    /// Peer card V11
    V11(),
}

impl NetworkPeer {
    /// Get peer card version
    pub fn version(&self) -> u32 {
        match *self {
            NetworkPeer::V10(ref _peer_v10) => 10,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Get peer card blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            NetworkPeer::V10(ref peer_v10) => peer_v10.blockstamp,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Get peer card issuer
    pub fn issuer(&self) -> PubKey {
        match *self {
            NetworkPeer::V10(ref peer_v10) => peer_v10.issuer,
            _ => panic!("Peer version is not supported !"),
        }
    }
    /// Verify validity of peer card signature
    pub fn verify(&self) -> bool {
        false
    }
    /// Get peer card endpoint
    pub fn get_endpoints(&self) -> Vec<NetworkEndpoint> {
        Vec::with_capacity(0)
    }
}

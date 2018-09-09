//  Copyright (C) 2017  The Durs Project Developers.
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

//! Module defining the format of network heads v3 and how to handle them.

use duniter_crypto::keys::*;
use duniter_documents::Blockstamp;
use std::cmp::Ordering;
use NodeId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3
pub struct NetworkHeadV3Container {
    /// Head step
    pub step: u8,
    /// head body
    pub body: NetworkHeadV3,
}

impl PartialOrd for NetworkHeadV3Container {
    fn partial_cmp(&self, other: &NetworkHeadV3Container) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NetworkHeadV3Container {
    fn cmp(&self, other: &NetworkHeadV3Container) -> Ordering {
        self.body.cmp(&other.body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Head V3
pub struct NetworkHeadV3 {
    ///
    pub api_outgoing_conf: u8,
    ///
    pub api_incoming_conf: u8,
    /// Issuer node free mirror rooms
    pub free_mirror_rooms: u8,
    /// Issuer node free "low priority" rooms
    pub low_priority_rooms: u8,
    /// Issuer node id
    pub node_id: NodeId,
    /// Issuer pubkey
    pub pubkey: PubKey,
    /// Head blockstamp
    pub blockstamp: Blockstamp,
    /// Issuer node software
    pub software: String,
    /// Issuer node soft version
    pub soft_version: String,
    /// Issuer signature
    pub signature: Option<Sig>,
}

impl PartialOrd for NetworkHeadV3 {
    fn partial_cmp(&self, other: &NetworkHeadV3) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NetworkHeadV3 {
    fn cmp(&self, other: &NetworkHeadV3) -> Ordering {
        self.blockstamp.cmp(&other.blockstamp)
    }
}

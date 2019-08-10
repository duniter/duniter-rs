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

//! Defined network events.

use crate::network_head::NetworkHead;
use crate::network_peer::PeerCard;
use crate::NodeFullId;
use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::UserDocumentDUBP;
use dubp_documents::Blockstamp;
use durs_common_tools::Percent;

#[derive(Debug, Clone)]
/// Type containing a network event, each time a network event occurs it's relayed to all modules
pub enum NetworkEvent {
    /// A connection has changed state(`u32` is the new state, `Option<String>` est l'uid du noeud)
    ConnectionStateChange(NodeFullId, u32, Option<String>, String),
    /// Generate new self peer card
    NewSelfPeer(PeerCard),
    /// Receiving Pending Documents
    ReceiveDocuments(Vec<UserDocumentDUBP>),
    /// Receiving blocks
    ReceiveBlocks(Vec<BlockDocument>),
    /// Receipt of peer cards
    ReceivePeers(Vec<PeerCard>),
    /// Receiving heads
    ReceiveHeads(Vec<NetworkHead>),
    /// Synchronisation event
    SyncEvent(SyncEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Sync event
pub enum SyncEvent {
    /// Target blockstamp
    ReceiveTargetBlockstamp(Blockstamp),
    /// Chunks size
    ReceiveChunksSize(usize),
    /// bars progression change
    BarsProgressionChange {
        /// Milestones percent
        milestones: Percent,
        /// Download percent
        download: Percent,
    },
    /// Receive correct blocks chunk
    ReceiveCorrectBlocksChunk {
        /// Blocks
        blocks: Vec<BlockDocument>,
        /// Raw blocks (compressed and serialized)
        raw_blocks: Option<Vec<u8>>,
    },
}

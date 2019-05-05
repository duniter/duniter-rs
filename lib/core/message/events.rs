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

use crate::*;
use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::DUBPDocument;
use dubp_documents::Blockstamp;
use durs_network::events::NetworkEvent;

/// The DURS event message.
#[derive(Debug, Clone)]
pub enum DursEvent {
    /// Arbitrary datas.
    ArbitraryDatas(ArbitraryDatas),
    /// Blockchain event
    BlockchainEvent(Box<BlockchainEvent>),
    /// MemPool Event (local node find next block)
    MemPoolEvent(MemPoolEvent),
    /// Network event
    NetworkEvent(NetworkEvent),
    /// Client API event
    ReceiveValidDocsFromClient(Vec<DUBPDocument>),
}

#[derive(Debug, Clone)]
/// MemPool module events
pub enum MemPoolEvent {
    /// FindNextBlock (local node find next block)
    FindNextBlock(Box<BlockDocument>),
    /// Store new Blockhain Document in Pool
    StoreNewDocInPool(Box<DUBPDocument>),
}

#[derive(Debug, Clone)]
/// Blockchain module events
pub enum BlockchainEvent {
    /// Stack up new valid block in local blockchain
    StackUpValidBlock(Box<BlockDocument>),
    /// Revert blocks in local blockchain
    RevertBlocks(Vec<BlockDocument>),
    /// Receive new valid pending document
    NewValidPendingDoc(DUBPDocument),
    /// Receive new refused pending document
    RefusedPendingDoc(DUBPDocument),
    /// Receive new refused pending block
    RefusedBlock(Blockstamp),
}

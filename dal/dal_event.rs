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

use dubp_documents::v10::block::BlockDocument;
use dubp_documents::*;

#[derive(Debug, Clone)]
/// Event to be transmitted to the other modules
pub enum DALEvent {
    /// Stack up new valid block in local blockchain
    StackUpValidBlock(Box<BlockDocument>, Blockstamp),
    /// Revert blocks in local blockchain
    RevertBlocks(Vec<Box<BlockDocument>>),
    /// Receive new valid pending document
    NewValidPendingDoc(DUBPDocument),
    /// Receive new refused pending document
    RefusedPendingDoc(DUBPDocument),
}

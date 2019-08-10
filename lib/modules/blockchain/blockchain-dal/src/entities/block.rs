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

use crate::*;
use dubp_documents::documents::block::{BlockDocument, BlockDocumentTrait};
use dubp_documents::Document;
use dubp_documents::{BlockNumber, Blockstamp};
use durs_wot::NodeId;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A block as it is saved in a database
pub struct DALBlock {
    /// Block document
    pub block: BlockDocument,
    /// List of certifications that expire in this block.
    /// Warning : BlockNumber contain the emission block, not the written block !
    /// HashMap<(Source, Target), BlockNumber>
    pub expire_certs: Option<HashMap<(NodeId, NodeId), BlockNumber>>,
}

impl DALBlock {
    /// Get blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        self.block.blockstamp()
    }
    /// Get previous blockstamp
    pub fn previous_blockstamp(&self) -> PreviousBlockstamp {
        self.block.previous_blockstamp()
    }
}

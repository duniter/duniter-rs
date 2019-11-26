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

//! Define entity CurrentUdDb

use dubp_block_doc::BlockDocument;
use dubp_common_doc::BlockNumber;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CurrentUdDb {
    pub amount: usize,
    pub base: usize,
    pub block_number: BlockNumber,
    pub members_count: usize,
    pub monetary_mass: usize,
    pub common_time: u64,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CurrentUdDbInternal {
    current: Option<CurrentUdDb>,
    previous: Option<CurrentUdDb>,
}

impl Into<Option<CurrentUdDb>> for CurrentUdDbInternal {
    fn into(self) -> Option<CurrentUdDb> {
        self.current
    }
}

impl CurrentUdDbInternal {
    pub fn update(&mut self, block_doc: &BlockDocument) {
        let BlockDocument::V10(ref block_doc_v10) = block_doc;
        if let Some(dividend) = block_doc_v10.dividend {
            self.previous = self.current;
            self.current = Some(CurrentUdDb {
                amount: dividend,
                base: block_doc_v10.unit_base,
                block_number: block_doc_v10.number,
                members_count: block_doc_v10.members_count,
                monetary_mass: block_doc_v10.monetary_mass,
                common_time: block_doc_v10.median_time,
            })
        }
    }
    pub fn revert(&mut self) {
        self.current = self.previous;
        self.previous = None;
    }
}

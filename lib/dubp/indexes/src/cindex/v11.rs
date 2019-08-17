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

//! Provides the definition of the certification index (CINDEX) described in the DUBP RFC v11.

use crate::{Index, IndexLineOp, MergeIndexLine};
use dubp_common_doc::blockstamp::Blockstamp;
use dup_crypto::keys::{PubKey, Sig};

/// CINDEX datas
pub type CIndexV11 = Index<(PubKey, PubKey), CIndexV11Line>;

#[derive(Clone, Copy, Debug)]
/// CINDEX line
pub struct CIndexV11Line {
    op: IndexLineOp,
    issuer: PubKey,
    receiver: PubKey,
    created_on: Option<Blockstamp>,
    written_on: Option<Blockstamp>,
    sig: Option<Sig>,
    expires_on: Option<u64>,
    expired_on: u64,
    chainable_on: Option<u64>,
    replayable_on: Option<u64>,
}

impl MergeIndexLine for CIndexV11Line {
    fn merge_index_line(&mut self, index_line: Self) {
        self.op = index_line.op;
        self.issuer = index_line.issuer;
        self.receiver = index_line.receiver;
        index_line.created_on.map(|v| self.created_on.replace(v));
        index_line.written_on.map(|v| self.written_on.replace(v));
        index_line.sig.map(|v| self.sig.replace(v));
        index_line.expires_on.map(|v| self.expires_on.replace(v));
        self.expired_on = index_line.expired_on;
        index_line
            .chainable_on
            .map(|v| self.chainable_on.replace(v));
        index_line
            .replayable_on
            .map(|v| self.replayable_on.replace(v));
    }
}

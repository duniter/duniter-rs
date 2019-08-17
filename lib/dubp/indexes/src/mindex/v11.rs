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

//! Provides the definition of the membership index (MINDEX) described in the DUBP RFC v11.

use crate::{Index, IndexLineOp, MergeIndexLine};
use dubp_common_doc::blockstamp::Blockstamp;
use dup_crypto::keys::{PubKey, Sig};

/// MINDEX datas
pub type MIndexV11 = Index<PubKey, MIndexV11Line>;

#[derive(Clone, Copy, Debug)]
/// MINDEX line
///
/// computed fields :
/// -
pub struct MIndexV11Line {
    op: IndexLineOp,
    r#pub: PubKey,
    created_on: Option<Blockstamp>,
    written_on: Blockstamp,
    expires_on: Option<u64>,
    expired_on: Option<u64>,
    revokes_on: Option<u64>,
    revoked_on: Option<Blockstamp>,
    leaving: Option<bool>,
    revocation: Option<Sig>,
    chainable_on: Option<u64>,
}

impl MergeIndexLine for MIndexV11Line {
    fn merge_index_line(&mut self, index_line: Self) {
        self.op = index_line.op;
        index_line.created_on.map(|v| self.created_on.replace(v));
        self.written_on = index_line.written_on;
        index_line.expires_on.map(|v| self.expires_on.replace(v));
        index_line.expired_on.map(|v| self.expired_on.replace(v));
        index_line.revokes_on.map(|v| self.revokes_on.replace(v));
        index_line.revoked_on.map(|v| self.revoked_on.replace(v));
        index_line.leaving.map(|v| self.leaving.replace(v));
        index_line.revocation.map(|v| self.revocation.replace(v));
        index_line
            .chainable_on
            .map(|v| self.chainable_on.replace(v));
    }
}

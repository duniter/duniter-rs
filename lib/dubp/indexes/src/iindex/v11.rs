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

//! Provides the definition of the identity index (IINDEX) described in the DUBP RFC v11.

use crate::iindex::Username;
use crate::{Index, IndexLineOp, MergeIndexLine};
use dubp_common_doc::blockstamp::Blockstamp;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::{PubKey, Sig};

/// IINDEX datas
pub type IIndexV11 = Index<PubKey, IIndexV11Line>;

#[derive(Clone, Copy, Debug)]
/// IINDEX line
///
/// computed fields :
/// - wasMember: NULL if kick=1 or member=0, 1 otherwise
pub struct IIndexV11Line {
    op: IndexLineOp,
    uid: Option<Username>,
    r#pub: PubKey,
    hash: Option<Hash>, // sha256(uid ++ pub ++ created_on)
    sig: Option<Sig>,
    created_on: Option<Blockstamp>,
    written_on: Blockstamp,
    member: Option<bool>,
    kick: Option<bool>,
}

impl MergeIndexLine for IIndexV11Line {
    fn merge_index_line(&mut self, index_line: Self) {
        self.op = index_line.op;
        index_line.uid.map(|v| self.uid.replace(v));
        index_line.hash.map(|v| self.hash.replace(v));
        index_line.sig.map(|v| self.sig.replace(v));
        index_line.created_on.map(|v| self.created_on.replace(v));
        self.written_on = index_line.written_on;
        index_line.sig.map(|v| self.sig.replace(v));
        index_line.member.map(|v| self.member.replace(v));
        index_line.kick.map(|v| self.kick.replace(v));
    }
}

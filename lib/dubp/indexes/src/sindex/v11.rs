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

//! Provides the definition of the source index (SINDEX) described in the DUBP RFC v11.

use super::SourceUniqueIdV10;
use crate::{Index, IndexLineOp, MergeIndexLine};
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_user_docs::documents::transaction::{TxAmount, TxBase, UTXOConditions};
use dup_crypto::hashs::Hash;

/// SINDEX datas
pub type SIndexV11 = Index<SourceUniqueIdV10, SIndexV11Line>;

#[derive(Clone, Debug)]
/// SINDEX line
///
/// computed fields :
/// - consumed: true if op == UPDATE, false otherwise.
pub struct SIndexV11Line {
    op: IndexLineOp,
    tx: Option<Hash>,
    identifier_and_pos: SourceUniqueIdV10,
    created_on: Option<Blockstamp>,
    amount: TxAmount,
    base: TxBase,
    locktime: usize,
    conditions: UTXOConditions,
    written_on: Blockstamp,
}

impl MergeIndexLine for SIndexV11Line {
    fn merge_index_line(&mut self, index_line: Self) {
        self.op = index_line.op;
        index_line.tx.map(|v| self.tx.replace(v));
        self.identifier_and_pos = index_line.identifier_and_pos;
        index_line.created_on.map(|v| self.created_on.replace(v));
        self.amount = index_line.amount;
        self.base = index_line.base;
        self.locktime = index_line.locktime;
        self.conditions = index_line.conditions;
        self.written_on = index_line.written_on;
    }
}

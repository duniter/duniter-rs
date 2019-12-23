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

#[derive(Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {

    use super::*;
    use dubp_common_doc::{BlockHash, BlockNumber};
    use dubp_user_docs::documents::transaction::{TransactionOutputCondition, UTXOConditionsGroup};
    use dup_crypto::keys::PubKey;

    #[test]
    fn test_iindex_merge_2_lines() {
        let cond = UTXOConditions {
            origin_str: None,
            conditions: UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(
                PubKey::default(),
            )),
        };
        let mut line1 = SIndexV11Line {
            op: IndexLineOp(true),
            tx: None,
            identifier_and_pos: SourceUniqueIdV10::UD(PubKey::default(), BlockNumber(0)),
            created_on: Some(Blockstamp::default()),
            amount: TxAmount(10),
            base: TxBase(0),
            locktime: 0,
            conditions: cond.clone(),
            written_on: Blockstamp::default(),
        };
        let b1 = Blockstamp {
            id: BlockNumber(1),
            hash: BlockHash(Hash::default()),
        };
        let line2 = SIndexV11Line {
            op: IndexLineOp(false),
            tx: None,
            identifier_and_pos: SourceUniqueIdV10::UD(PubKey::default(), BlockNumber(0)),
            created_on: None,
            amount: TxAmount(10),
            base: TxBase(0),
            locktime: 0,
            conditions: cond.clone(),
            written_on: b1,
        };
        line1.merge_index_line(line2);
        assert_eq!(
            line1,
            SIndexV11Line {
                op: IndexLineOp(false),
                tx: None,
                identifier_and_pos: SourceUniqueIdV10::UD(PubKey::default(), BlockNumber(0)),
                created_on: Some(Blockstamp::default()),
                amount: TxAmount(10),
                base: TxBase(0),
                locktime: 0,
                conditions: cond,
                written_on: b1,
            }
        )
    }
}

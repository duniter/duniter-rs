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

//! Rule BR_G03 - previousIssuer

use super::{InvalidRuleError, RuleDatas, RuleNotSyncDatas};
use dubp_block_doc::BlockDocument;
use durs_bc_db_reader::BcDbInReadTx;
use durs_common_tools::traits::bool_ext::BoolExt;
use rules_engine::rule::{Rule, RuleFn, RuleNumber};
use rules_engine::ProtocolVersion;
use unwrap::unwrap;

#[inline]
pub fn rule<'d, 'db, DB: BcDbInReadTx>(
) -> Rule<RuleDatas<'d>, RuleNotSyncDatas<'db, DB>, InvalidRuleError> {
    unwrap!(Rule::new(
        RuleNumber(3),
        maplit::btreemap![
            ProtocolVersion(10) => RuleFn::Ref(v10),
        ]
    ))
}

fn v10(rule_datas: &RuleDatas) -> Result<(), InvalidRuleError> {
    let RuleDatas {
        ref block,
        ref previous_block,
        ..
    } = rule_datas;
    let BlockDocument::V10(ref block) = block;
    let BlockDocument::V10(ref previous_block) = previous_block;

    (Some(previous_block.issuers[0]) == block.previous_issuer)
        .or_err(InvalidRuleError::WrongPreviousIssuer)?;

    Ok(())
}

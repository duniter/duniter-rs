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

//! Sub-module define all rules of blockchain protocol.

use super::br_g03;
use super::br_g100;
use super::{RuleDatas, RuleNotSyncDatas};
use crate::dubp::check::global::rules::InvalidRuleError;
use durs_bc_db_reader::BcDbInReadTx;
use rules_engine::rule::{Rule, RuleNumber};
use std::collections::BTreeMap;

#[inline]
pub fn get_all_rules<'d, 'db, DB: BcDbInReadTx>(
) -> BTreeMap<RuleNumber, Rule<RuleDatas<'d>, RuleNotSyncDatas<'db, DB>, InvalidRuleError>> {
    maplit::btreemap![
        RuleNumber(3) => br_g03::rule(),
        RuleNumber(100) => br_g100::rule(),
    ]
}

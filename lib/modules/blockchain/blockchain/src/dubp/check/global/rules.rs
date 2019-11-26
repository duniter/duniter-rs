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

//! Sub-module define rules engine

pub mod all_rules;
mod br_g03;
mod br_g100;

use dubp_block_doc::BlockDocument;
//use dup_crypto::keys::PubKey;
use durs_bc_db_reader::indexes::identities::IdentityStateDb;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
//use durs_wot::*;
use failure::Fail;
//use std::collections::HashMap;

#[derive(Debug)]
pub struct RuleDatas<'a> {
    pub(crate) block: &'a BlockDocument,
    pub(crate) previous_block: &'a BlockDocument,
    //db: &'a Db,
    //wot_db: &BinFreeStructDb<W>,
    //wot_index: HashMap<PubKey, NodeId>,
    //current_frame: Option<HashMap<PubKey, usize>>,
}

pub struct RuleNotSyncDatas<'db, DB: BcDbInReadTx> {
    pub(crate) db: &'db DB,
}

#[derive(Clone, Debug, Eq, Fail, PartialEq)]
pub enum InvalidRuleError {
    #[fail(display = "Database error: {:?}", _0)]
    DbError(String),
    #[fail(display = "BR_G99: different currency")]
    _DifferentCurrency,
    #[fail(display = "BR_G03: wrong previous issuer")]
    WrongPreviousIssuer,
    #[fail(display = "BR_G100: issuer is not a member (not exist)")]
    IssuerNotExist,
    #[fail(display = "BR_G100: issuer is not a member (issuer_state={:?})", _0)]
    NotMemberIssuer(IdentityStateDb),
    #[fail(display = "BR_G04: wrong issuers count")]
    _WrongIssuersCount,
    #[fail(display = "BR_G05: wrong issuers frame size")]
    _WrongIssuersFrame,
}

impl From<DbError> for InvalidRuleError {
    fn from(e: DbError) -> Self {
        Self::DbError(format!("{:?}", e))
    }
}

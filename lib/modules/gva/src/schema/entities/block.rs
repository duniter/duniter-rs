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

// ! Module define graphql Block type

use crate::context::QueryContext;
use crate::schema::query_trails::QueryTrailBlockExtensions;
use chrono::NaiveDateTime;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
use durs_common_tools::fatal_error;
use juniper::{Executor, FieldResult};
use juniper_from_schema::{QueryTrail, Walked};

pub struct Block {
    version: i32,
    currency: String,
    issuer: String,
    issuer_name: Option<String>,
    issuers_count: i32,
    members_count: i32,
    number: i32,
    hash: String,
    common_time: NaiveDateTime,
    pow_min: i32,
}

impl Block {
    #[inline]
    pub(crate) fn ask_field_issuer_name(trail: &QueryTrail<'_, Block, Walked>) -> bool {
        trail.issuer_name()
    }
    // Convert BlockDb (db entity) into Block (gva entity)
    pub(crate) fn from_block_db<DB: BcDbInReadTx>(
        db: &DB,
        block_db: BlockDb,
        ask_issuer_name: bool,
    ) -> Result<Block, DbError> {
        Ok(Block {
            version: block_db.block.version() as i32,
            currency: block_db.block.currency().to_string(),
            issuer: block_db.block.issuers()[0].to_string(),
            issuer_name: if ask_issuer_name {
                db.get_uid_from_pubkey(&block_db.block.issuers()[0])?
            } else {
                None
            },
            issuers_count: block_db.block.issuers_count() as i32,
            members_count: block_db.block.members_count() as i32,
            number: block_db.block.number().0 as i32,
            hash: block_db
                .block
                .hash()
                .unwrap_or_else(|| fatal_error!("BlockDb without hash."))
                .to_string(),
            common_time: NaiveDateTime::from_timestamp(block_db.block.common_time() as i64, 0),
            pow_min: block_db.block.pow_min() as i32,
        })
    }
}

impl super::super::BlockFields for Block {
    #[inline]
    fn field_version(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.version)
    }
    #[inline]
    fn field_currency(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&String> {
        Ok(&self.currency)
    }
    #[inline]
    fn field_issuer(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&String> {
        Ok(&self.issuer)
    }
    #[inline]
    fn field_issuer_name(
        &self,
        _executor: &Executor<'_, QueryContext>,
    ) -> FieldResult<&Option<String>> {
        Ok(&self.issuer_name)
    }
    #[inline]
    fn field_number(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.number)
    }
    #[inline]
    fn field_hash(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&String> {
        Ok(&self.hash)
    }
    #[inline]
    fn field_common_time(
        &self,
        _executor: &Executor<'_, QueryContext>,
    ) -> FieldResult<&NaiveDateTime> {
        Ok(&self.common_time)
    }
    #[inline]
    fn field_pow_min(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.pow_min)
    }
    #[inline]
    fn field_issuers_count(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.issuers_count)
    }
    #[inline]
    fn field_members_count(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.members_count)
    }
}

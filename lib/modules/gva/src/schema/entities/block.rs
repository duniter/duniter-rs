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

use crate::context::Context;
use chrono::NaiveDateTime;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::blocks::DbBlock;
use durs_common_tools::fatal_error;
use juniper::{Executor, FieldResult};

pub struct Block {
    version: i32,
    currency: String,
    issuer: String,
    number: i32,
    hash: String,
    common_time: NaiveDateTime,
}

impl super::super::BlockFields for Block {
    fn field_version(&self, _executor: &Executor<'_, Context>) -> FieldResult<&i32> {
        Ok(&self.version)
    }

    fn field_currency(&self, _executor: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.currency)
    }

    fn field_issuer(&self, _executor: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.issuer)
    }

    fn field_number(&self, _executor: &Executor<'_, Context>) -> FieldResult<&i32> {
        Ok(&self.number)
    }

    fn field_hash(&self, _executor: &Executor<'_, Context>) -> FieldResult<&String> {
        Ok(&self.hash)
    }

    fn field_common_time(&self, _executor: &Executor<'_, Context>) -> FieldResult<&NaiveDateTime> {
        Ok(&self.common_time)
    }
}

impl Block {
    pub fn from_db_block(db_block: DbBlock) -> Block {
        Block {
            version: db_block.block.version() as i32,
            currency: db_block.block.currency().to_string(),
            issuer: db_block.block.issuers()[0].to_string(),
            number: db_block.block.number().0 as i32,
            hash: db_block
                .block
                .hash()
                .unwrap_or_else(|| fatal_error!("DbBlock without hash."))
                .to_string(),
            common_time: NaiveDateTime::from_timestamp(db_block.block.common_time() as i64, 0),
        }
    }
}

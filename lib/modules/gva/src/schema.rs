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
//
// model and resolvers implementation

use crate::context::Context;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::BcDbRo;
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::graphql_schema_from_file;

// generate schema from schema file
graphql_schema_from_file!("resources/schema.gql");

pub struct Query;

pub struct Block {
    version: i32,
    currency: String,
    issuer: String,
    number: i32,
}

impl QueryFields for Query {
    fn field_current(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Block, Walked>,
    ) -> FieldResult<Option<Block>> {
        let db: &BcDbRo = &_executor.context().get_db();
        let current_blockstamp = durs_bc_db_reader::current_meta_datas::get_current_blockstamp(db);

        match current_blockstamp {
            Ok(option) => match option {
                Some(v) => {
                    let current_block = durs_bc_db_reader::blocks::get_block(db, v);
                    match current_block {
                        Ok(current_block_option) => match current_block_option {
                            Some(block) => Ok(Some(Block {
                                version: block.block.version() as i32,
                                currency: block.block.currency().to_string(),
                                issuer: block.block.issuers()[0].to_string(),
                                number: block.block.number().0 as i32,
                            })),
                            None => Ok(None),
                        },
                        Err(_e) => Err(juniper::FieldError::from("No current block available")),
                    }
                }
                None => Ok(None),
            },
            Err(_e) => Err(juniper::FieldError::from("No current block available")),
        }
    }
}

impl BlockFields for Block {
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
}

pub struct Mutation;

impl MutationFields for Mutation {
    fn field_noop(&self, _executor: &Executor<'_, Context>) -> FieldResult<&bool> {
        Ok(&true)
    }
}

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}

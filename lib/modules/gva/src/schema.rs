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

// ! model and resolvers implementation

mod block;
mod paging;

use self::block::Block;
use crate::context::Context;
use dubp_common_doc::BlockNumber;
use durs_bc_db_reader::{BcDbRo, DbError, DbReadable};
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::graphql_schema_from_file;

// generate schema from schema file
graphql_schema_from_file!("resources/schema.gql");

pub struct Query;

pub struct Summary {
    software: &'static str,
    version: &'static str,
}

pub struct Node {
    summary: Summary,
}

fn db_err_to_juniper_err(e: DbError) -> juniper::FieldError {
    juniper::FieldError::from(format!("Db error: {:?}", e))
}

impl QueryFields for Query {
    fn field_node(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Node, Walked>,
    ) -> FieldResult<Node> {
        Ok(Node {
            summary: Summary {
                software: executor.context().get_software_name(),
                version: executor.context().get_software_version(),
            },
        })
    }
    fn field_current(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Block, Walked>,
    ) -> FieldResult<Option<Block>> {
        let db: &BcDbRo = &executor.context().get_db();

        db.read(|r| {
            if let Some(current_blockstamp) =
                durs_bc_db_reader::current_meta_datas::get_current_blockstamp_(db, r)?
            {
                block::get_block(db, r, current_blockstamp.id)
            } else {
                Ok(None)
            }
        })
        .map_err(db_err_to_juniper_err)
    }
    fn field_block(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Block, Walked>,
        number: i32,
    ) -> FieldResult<Option<Block>> {
        let db: &BcDbRo = &executor.context().get_db();

        let block_number = if number >= 0 {
            BlockNumber(number as u32)
        } else {
            return Err(juniper::FieldError::from("Block number must be positive."));
        };

        db.read(|r| block::get_block(db, r, block_number))
            .map_err(db_err_to_juniper_err)
    }
    fn field_blocks(
        &self,
        executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Block, Walked>,
        paging_opt: Option<Paging>,
    ) -> FieldResult<Vec<Block>> {
        let db: &BcDbRo = &executor.context().get_db();
        db.read(|r| {
            paging::FilledPaging::new(db, r, paging_opt)?
                .get_range()
                .filter_map(|n| match block::get_block(db, r, BlockNumber(n)) {
                    Ok(Some(db_block)) => Some(Ok(db_block)),
                    Ok(None) => None,
                    Err(e) => Some(Err(e)),
                })
                .collect::<Result<Vec<Block>, DbError>>()
        })
        .map_err(db_err_to_juniper_err)
    }
}

impl NodeFields for Node {
    fn field_summary(
        &self,
        _executor: &Executor<'_, Context>,
        _trail: &QueryTrail<'_, Summary, Walked>,
    ) -> &Summary {
        &self.summary
    }
}

impl SummaryFields for Summary {
    fn field_software(&self, _executor: &Executor<'_, Context>) -> String {
        self.software.to_owned()
    }
    fn field_version(&self, _executor: &Executor<'_, Context>) -> String {
        self.version.to_owned()
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

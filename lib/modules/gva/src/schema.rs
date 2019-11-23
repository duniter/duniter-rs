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

// ! Module define GraphQl schema

mod entities;
pub mod inputs;
mod queries;

use self::entities::block::Block;
use self::entities::node::{Node, Summary};
use crate::context::QueryContext;
#[cfg(not(test))]
use durs_bc_db_reader::{BcDbRoWithReader, DbReadable};
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::graphql_schema_from_file;

// generate schema from schema file
graphql_schema_from_file!("resources/schema.gql", context_type: QueryContext);

pub struct Query;

impl QueryFields for Query {
    #[inline]
    fn field_api_version(&self, _executor: &Executor<'_, QueryContext>) -> &i32 {
        &crate::constants::API_VERSION
    }
    #[inline]
    fn field_node(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, Node, Walked>,
    ) -> FieldResult<Node> {
        queries::node::execute(executor.context(), trail)
    }
    #[inline]
    fn field_current(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, Block, Walked>,
    ) -> FieldResult<Option<Block>> {
        let db = executor.context().get_db();
        cfg_if::cfg_if! {
            if #[cfg(not(test))] {
                db.read(|r| queries::current::execute(&BcDbRoWithReader { db, r }, trail)).map_err(Into::into)
            } else {
                queries::current::execute(db, trail).map_err(Into::into)
            }
        }
    }
    #[inline]
    fn field_block(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, Block, Walked>,
        number: i32,
    ) -> FieldResult<Option<Block>> {
        let db = executor.context().get_db();
        cfg_if::cfg_if! {
            if #[cfg(not(test))] {
                db.read(|r| queries::block::execute(&BcDbRoWithReader { db, r }, trail, number)).map_err(Into::into)
            } else {
                queries::block::execute(db, trail, number).map_err(Into::into)
            }
        }
    }
    #[inline]
    fn field_blocks(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, Block, Walked>,
        block_interval_opt: Option<BlockInterval>,
        paging_opt: Option<Paging>,
        mut step: i32,
    ) -> FieldResult<Vec<Block>> {
        if step <= 0 {
            step = 1;
        }
        let db = executor.context().get_db();
        cfg_if::cfg_if! {
            if #[cfg(not(test))] {
                db.read(|r| {
                    queries::blocks::execute(
                        &BcDbRoWithReader { db, r },
                        trail,
                        paging_opt,
                        block_interval_opt,
                        step as usize,
                    )
                })
                .map_err(Into::into)
            } else {
                queries::blocks::execute(
                    db,
                    trail,
                    paging_opt,
                    block_interval_opt,
                    step as usize,
                )
                .map_err(Into::into)
            }
        }
    }
}

pub struct Mutation;

impl MutationFields for Mutation {
    fn field_noop(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&bool> {
        Ok(&true)
    }
}

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}

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
use self::entities::blocks_page::BlocksPage;
use self::entities::current_ud::CurrentUd;
use self::entities::node::{Node, Summary};
use crate::context::QueryContext;
#[cfg(not(test))]
use durs_bc_db_reader::{BcDbRoWithReader, DbReadable};
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::graphql_schema_from_file;

// generate schema from schema file
graphql_schema_from_file!("resources/schema.gql", context_type: QueryContext);

/// Macro that execute a query resolver in db read transaction
#[cfg(not(test))]
macro_rules! exec_in_db_transaction {
    ($f:ident($e:ident, $($param:expr),*)) => {
        {
            let db = $e.context().get_db();
            db.read(|r| queries::$f::execute(&BcDbRoWithReader { db, r }$(, $param)*)).map_err(Into::into)
        }
    };
}
#[cfg(test)]
macro_rules! exec_in_db_transaction {
    ($f:ident($e:ident, $($param:expr),*)) => {
        {
            let db = $e.context().get_db();
            queries::$f::execute(db$(, $param)*).map_err(Into::into)
        }
    };
}

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
        exec_in_db_transaction!(current(executor, trail))
    }
    #[inline]
    fn field_block(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, Block, Walked>,
        number: i32,
    ) -> FieldResult<Option<Block>> {
        exec_in_db_transaction!(block(executor, trail, number))
    }
    #[inline]
    fn field_blocks(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, BlocksPage, Walked>,
        block_interval_opt: Option<BlockInterval>,
        paging_opt: Option<Paging>,
        mut step: i32,
        sort_order: SortOrder,
    ) -> FieldResult<BlocksPage> {
        if step <= 0 {
            step = 1;
        }
        exec_in_db_transaction!(blocks(
            executor,
            trail,
            paging_opt.as_ref(),
            block_interval_opt.as_ref(),
            step as usize,
            sort_order
        ))
    }
    #[inline]
    fn field_current_ud(
        &self,
        executor: &Executor<'_, QueryContext>,
        trail: &QueryTrail<'_, CurrentUd, Walked>,
    ) -> FieldResult<Option<CurrentUd>> {
        exec_in_db_transaction!(current_ud(executor, trail))
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

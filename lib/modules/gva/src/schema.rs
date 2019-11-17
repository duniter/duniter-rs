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
mod paging;
mod queries;

use self::entities::block::Block;
use self::entities::node::{Node, Summary};
use crate::context::Context;
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::graphql_schema_from_file;

// generate schema from schema file
graphql_schema_from_file!("resources/schema.gql");

pub struct Query;

impl QueryFields for Query {
    #[inline]
    fn field_node(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, Node, Walked>,
    ) -> FieldResult<Node> {
        queries::node::execute(executor, trail)
    }
    #[inline]
    fn field_current(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, Block, Walked>,
    ) -> FieldResult<Option<Block>> {
        queries::current::execute(executor, trail)
    }
    #[inline]
    fn field_block(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, Block, Walked>,
        number: i32,
    ) -> FieldResult<Option<Block>> {
        queries::block::execute(executor, trail, number)
    }
    #[inline]
    fn field_blocks(
        &self,
        executor: &Executor<'_, Context>,
        trail: &QueryTrail<'_, Block, Walked>,
        paging_opt: Option<Paging>,
    ) -> FieldResult<Vec<Block>> {
        queries::blocks::execute(executor, trail, paging_opt)
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

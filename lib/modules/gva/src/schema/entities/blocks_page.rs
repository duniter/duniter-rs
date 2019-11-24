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

// ! Module define graphql BlocksPage type

use crate::context::QueryContext;
use crate::schema::entities::block::Block;
use crate::schema::query_trails::QueryTrailBlocksPageExtensions;
use juniper::{Executor, FieldResult};
use juniper_from_schema::{QueryTrail, Walked};

pub struct BlocksPage {
    pub(crate) blocks: Vec<Block>,
    pub(crate) current_page_number: i32,
    pub(crate) interval_from: i32,
    pub(crate) interval_to: i32,
    pub(crate) last_page_number: i32,
    pub(crate) total_blocks_count: i32,
}

impl BlocksPage {
    pub(crate) fn ask_field_blocks_issuer_name(trail: &QueryTrail<'_, BlocksPage, Walked>) -> bool {
        if let Some(block_trail) = trail.blocks().walk() {
            Block::ask_field_issuer_name(&block_trail)
        } else {
            false
        }
    }
}

impl super::super::BlocksPageFields for BlocksPage {
    #[inline]
    fn field_blocks(
        &self,
        _executor: &Executor<'_, QueryContext>,
        _trail: &QueryTrail<'_, Block, Walked>,
    ) -> FieldResult<&Vec<Block>> {
        Ok(&self.blocks)
    }
    #[inline]
    fn field_current_page_number(
        &self,
        _executor: &Executor<'_, QueryContext>,
    ) -> FieldResult<&i32> {
        Ok(&self.current_page_number)
    }
    #[inline]
    fn field_interval_from(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.interval_from)
    }
    #[inline]
    fn field_interval_to(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.interval_to)
    }
    #[inline]
    fn field_last_page_number(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.last_page_number)
    }
    #[inline]
    fn field_total_blocks_count(
        &self,
        _executor: &Executor<'_, QueryContext>,
    ) -> FieldResult<&i32> {
        Ok(&self.total_blocks_count)
    }
}

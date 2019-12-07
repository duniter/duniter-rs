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

// ! Module define graphql BlockCurrent UD type
use crate::context::QueryContext;
use chrono::NaiveDateTime;
use durs_bc_db_reader::current_metadata::current_ud::CurrentUdDb;
use juniper::{Executor, FieldResult};

pub struct CurrentUd {
    pub amount: i32,
    pub base: i32,
    pub block_number: i32,
    pub blockchain_time: NaiveDateTime,
    pub members_count: i32,
    pub monetary_mass: i32,
}

impl CurrentUd {
    // Convert BloCurrentUdDb (db entity) into CurrentUd (gva entity)
    pub(crate) fn from_current_du_db(current_du_db: CurrentUdDb) -> CurrentUd {
        CurrentUd {
            amount: current_du_db.amount as i32,
            base: current_du_db.base as i32,
            block_number: current_du_db.block_number.0 as i32,
            blockchain_time: NaiveDateTime::from_timestamp(current_du_db.common_time as i64, 0),
            members_count: current_du_db.members_count as i32,
            monetary_mass: current_du_db.monetary_mass as i32,
        }
    }
}

impl super::super::CurrentUdFields for CurrentUd {
    #[inline]
    fn field_amount(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.amount)
    }
    #[inline]
    fn field_base(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.base)
    }
    #[inline]
    fn field_block_number(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.block_number)
    }
    #[inline]
    fn field_blockchain_time(
        &self,
        _executor: &Executor<'_, QueryContext>,
    ) -> FieldResult<&NaiveDateTime> {
        Ok(&self.blockchain_time)
    }
    #[inline]
    fn field_members_count(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.members_count)
    }
    #[inline]
    fn field_monetary_mass(&self, _executor: &Executor<'_, QueryContext>) -> FieldResult<&i32> {
        Ok(&self.monetary_mass)
    }
}

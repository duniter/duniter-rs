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

//! Context for graphql resolvers

use crate::db::BcDbRo;
use crate::schema::Schema;

pub struct GlobalContext {
    db: &'static BcDbRo,
    pub(crate) schema: Schema,
    software_name: &'static str,
    software_version: &'static str,
}

impl GlobalContext {
    pub(crate) fn new(
        db: &'static BcDbRo,
        schema: Schema,
        software_name: &'static str,
        software_version: &'static str,
    ) -> Self {
        GlobalContext {
            db,
            schema,
            software_name,
            software_version,
        }
    }
}

pub struct QueryContext {
    db: &'static BcDbRo,
    software_name: &'static str,
    software_version: &'static str,
}

impl juniper::Context for QueryContext {}

impl From<&GlobalContext> for QueryContext {
    fn from(global_context: &GlobalContext) -> Self {
        QueryContext {
            db: global_context.db,
            software_name: global_context.software_name,
            software_version: global_context.software_version,
        }
    }
}

impl QueryContext {
    pub(crate) fn get_db(&self) -> &BcDbRo {
        &self.db
    }

    pub fn get_software_name(&self) -> &'static str {
        &self.software_name
    }

    pub fn get_software_version(&self) -> &'static str {
        &self.software_version
    }
}

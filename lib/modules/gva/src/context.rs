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

#[cfg(not(test))]
use durs_bc_db_reader::BcDbRo;
use durs_common_tools::fatal_error;

#[cfg(test)]
use crate::db::MockBcDbTrait;

/// GVA context (access to database)
static mut CONTEXT: Option<Context> = None;

#[cfg(not(test))]
pub type DB = BcDbRo;
#[cfg(test)]
pub(crate) type DB = MockBcDbTrait;

pub struct Context {
    db: DB,
    software_name: &'static str,
    software_version: &'static str,
}

impl juniper::Context for Context {}

impl Context {
    pub(crate) fn new(db: DB, software_name: &'static str, software_version: &'static str) -> Self {
        Context {
            db,
            software_name,
            software_version,
        }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }

    pub fn get_software_name(&self) -> &'static str {
        &self.software_name
    }

    pub fn get_software_version(&self) -> &'static str {
        &self.software_version
    }
}

pub(crate) fn init(db: DB, soft_name: &'static str, soft_version: &'static str) {
    unsafe {
        CONTEXT.replace(Context::new(db, soft_name, soft_version));
    }
}

pub fn get_context() -> &'static Context {
    unsafe {
        if let Some(ref context) = CONTEXT {
            context
        } else {
            fatal_error!("GVA: no context");
        }
    }
}

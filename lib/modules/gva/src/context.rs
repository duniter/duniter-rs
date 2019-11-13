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

use durs_bc_db_reader::BcDbRo;
use durs_common_tools::fatal_error;

/// GVA context (access to database)
static mut CONTEXT: Option<Context> = None;

#[derive(Debug)]
pub struct Context {
    db: BcDbRo,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(db: BcDbRo) -> Self {
        Context { db }
    }

    pub fn get_db(&self) -> &BcDbRo {
        &self.db
    }
}

pub fn init(db: BcDbRo) {
    unsafe {
        CONTEXT.replace(Context::new(db));
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

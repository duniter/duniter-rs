//  Copyright (C) 2019  Éloïs SANCHEZ
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

//! Mocks for Dunitrust Blockchain DB

use dubp_block_doc::BlockDocument;
use durs_bc_db_reader::blocks::fork_tree::ForkTree;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_writer::blocks::{insert_new_fork_block, insert_new_head_block};
use durs_bc_db_writer::current_metadata::update_current_metadata;
use durs_bc_db_writer::{Db, DbError};

/// Warning : This function does not update the indexes and considers
/// that your block is valid (so chainable on the main chain).
/// To be used only for tests that do not use indexes.
/// To insert a fork block, use `insert_fork_block` instead.
pub fn insert_main_block(
    db_tmp: &Db,
    block: BlockDocument,
    fork_tree: Option<&mut ForkTree>,
) -> Result<(), DbError> {
    db_tmp.write(|mut w| {
        update_current_metadata(db_tmp, &mut w, &block)?;
        insert_new_head_block(
            &db_tmp,
            &mut w,
            fork_tree,
            BlockDb {
                block,
                expire_certs: None,
            },
        )?;
        Ok(w)
    })
}

/// Insert fork block
pub fn insert_fork_block(
    db_tmp: &Db,
    fork_tree: &mut ForkTree,
    block: BlockDocument,
) -> Result<bool, DbError> {
    let mut orphan = false;
    db_tmp.write(|mut w| {
        orphan = !insert_new_fork_block(
            db_tmp,
            &mut w,
            fork_tree,
            BlockDb {
                block,
                expire_certs: None,
            },
        )?;
        Ok(w)
    })?;
    Ok(orphan)
}

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

use dubp_block_doc::BlockDocument;
use dup_crypto::keys::{PubKey, PublicKey};
use durs_bc_db_reader::blocks::DbBlock;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::{DbReadable, DbValue};
use durs_bc_db_writer::DbWriter;
use durs_bc_db_writer::{Db, DbError};
use durs_wot::WotId;
use tempfile::tempdir;

#[inline]
/// Open database in an arbitrary temporary directory given by OS
/// and automatically cleaned when `Db` is dropped
pub fn open_tmp_db() -> Result<Db, DbError> {
    durs_bc_db_writer::open_db(tempdir().map_err(DbError::FileSystemError)?.path())
}

pub fn to_db_block(block: BlockDocument) -> DbBlock {
    DbBlock {
        block,
        expire_certs: None,
    }
}

pub fn insert_wot_index_entry(
    db: &Db,
    w: &mut DbWriter,
    wot_id: WotId,
    pubkey: PubKey,
) -> Result<(), DbError> {
    db.get_store(WOT_ID_INDEX).put(
        w.as_mut(),
        &pubkey.to_bytes_vector(),
        &DbValue::U64(wot_id.0 as u64),
    )?;
    Ok(())
}

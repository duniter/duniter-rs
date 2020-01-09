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

//! Crypto tests tools for projects use dup-crypto.

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

pub mod mocks;

use durs_bc_db_writer::{Db, DbError};
use tempfile::tempdir;

#[inline]
/// Open database in an arbitrary temporary directory given by OS
/// and automatically cleaned when `Db` is dropped
pub fn open_tmp_db() -> Result<Db, DbError> {
    Db::open_db(
        tempdir().map_err(DbError::FileSystemError)?.path(),
        &durs_bc_db_reader::bc_db_schema(),
    )
}

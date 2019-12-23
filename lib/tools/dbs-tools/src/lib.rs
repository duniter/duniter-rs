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

//! DBs tools for Dunitrust project.

#![allow(dead_code, unused_imports, clippy::large_enum_variant)]
#![deny(
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

mod errors;
mod free_struct_db;
pub mod kv_db;

pub use errors::DbError;
pub use free_struct_db::{open_free_struct_file_db, open_free_struct_memory_db, BinFreeStructDb};

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::default::Default;
use std::fmt::Debug;
use std::path::PathBuf;

/// Convert rust type to bytes
#[inline]
pub fn to_bytes<T: Serialize>(t: &T) -> Result<Vec<u8>, DbError> {
    Ok(bincode::serialize(t)?)
}

/// Open free structured database
pub fn open_free_struct_db<D: Serialize + DeserializeOwned + Debug + Default + Clone + Send>(
    dbs_folder_path: Option<&PathBuf>,
    db_file_name: &str,
) -> Result<BinFreeStructDb<D>, DbError> {
    if let Some(dbs_folder_path) = dbs_folder_path {
        Ok(BinFreeStructDb::File(open_free_struct_file_db::<D>(
            dbs_folder_path,
            db_file_name,
        )?))
    } else {
        Ok(BinFreeStructDb::Mem(open_free_struct_memory_db::<D>()?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_open_unexist_free_struct_db() {
        assert!(open_free_struct_db::<usize>(Some(&PathBuf::new()), "").is_err())
    }
}

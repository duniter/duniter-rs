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

//! Datas Access Layer

#![allow(clippy::large_enum_variant)]
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

#[macro_use]
extern crate log;

pub mod blocks;
pub mod current_meta_datas;
pub mod indexes;
pub mod store_name;
pub mod writers;

pub use durs_dbs_tools::kv_db::{
    KvFileDbHandler, KvFileDbRead as DbReadable, KvFileDbRoHandler, KvFileDbSchema,
    KvFileDbStoreType, KvFileDbValue, KvFileDbWriter as DbWriter,
};
pub use durs_dbs_tools::{
    open_free_struct_db, open_free_struct_file_db, open_free_struct_memory_db,
};
pub use durs_dbs_tools::{BinFreeStructDb, DbError};

use dubp_common_doc::{BlockNumber, Blockstamp};
use dubp_indexes::sindex::UniqueIdUTXOv10;
use dubp_user_docs::documents::transaction::*;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_wot::data::rusty::RustyWebOfTrust;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Database handler
pub type Db = KvFileDbHandler;

/// Read-only database handler
pub type DbReader = KvFileDbRoHandler;

/// Database containing the wot graph (each node of the graph in an u32)
pub type WotDB = RustyWebOfTrust;

/// Open database
#[inline]
pub fn open_db(path: &Path) -> Result<Db, DbError> {
    Db::open_db(path, &durs_bc_db_reader::bc_db_schema())
}

#[derive(Debug)]
/// Set of databases storing web of trust information
pub struct WotsV10DBs {
    /// Store wot graph
    pub wot_db: BinFreeStructDb<WotDB>,
}

impl WotsV10DBs {
    /// Open wot databases from their respective files
    pub fn open(db_path: Option<&PathBuf>) -> WotsV10DBs {
        WotsV10DBs {
            wot_db: open_free_struct_db::<RustyWebOfTrust>(db_path, "wot.db")
                .expect("Fail to open WotDB"),
        }
    }
    /// Save wot databases from their respective files
    pub fn save_dbs(&self) {
        info!("BC-DB-WRITER: Save WotsV10DBs.");
        self.wot_db
            .save()
            .expect("Fatal error : fail to save WotDB !");
    }
}

/*#[derive(Debug, Clone)]
pub struct WotStats {
    pub block_number: u32,
    pub block_hash: String,
    pub sentries_count: usize,
    pub average_density: usize,
    pub average_distance: usize,
    pub distances: Vec<usize>,
    pub average_connectivity: usize,
    pub connectivities: Vec<usize>,
    pub average_centrality: usize,
    pub centralities: Vec<u64>,
}*/

#[cfg(test)]
pub mod tests {

    use super::*;
    use tempfile::tempdir;

    #[inline]
    /// Open database in an arbitrary temporary directory given by OS
    /// and automatically cleaned when `Db` is dropped
    pub fn open_tmp_db() -> Result<Db, DbError> {
        open_db(tempdir().map_err(DbError::FileSystemError)?.path())
    }
}

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

//! BlockChain Datas Access Layer in Read-Only mode.

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

pub mod blocks;
pub mod constants;
pub mod currency_params;
pub mod current_frame;
pub mod current_meta_datas;
pub mod indexes;
pub mod paging;
pub mod tools;

pub use durs_dbs_tools::kv_db::{
    KvFileDbRead as DbReadable, KvFileDbRoHandler as BcDbRo, KvFileDbSchema, KvFileDbStoreType,
    KvFileDbValue as DbValue, Readable as DbReader,
};

use constants::*;
use durs_dbs_tools::DbError;
use maplit::hashmap;
use std::path::Path;

#[inline]
/// Get BlockChain DB Schema
pub fn bc_db_schema() -> KvFileDbSchema {
    KvFileDbSchema {
        stores: hashmap![
            CURRENT_METAS_DATAS.to_owned() => KvFileDbStoreType::SingleIntKey,
            MAIN_BLOCKS.to_owned() => KvFileDbStoreType::SingleIntKey,
            FORK_BLOCKS.to_owned() => KvFileDbStoreType::Single,
            ORPHAN_BLOCKSTAMP.to_owned() => KvFileDbStoreType::Single,
            IDENTITIES.to_owned() => KvFileDbStoreType::SingleIntKey,
            MBS_BY_CREATED_BLOCK.to_owned() => KvFileDbStoreType::MultiIntKey,
            CERTS_BY_CREATED_BLOCK.to_owned() => KvFileDbStoreType::MultiIntKey,
            WOT_ID_INDEX.to_owned() => KvFileDbStoreType::Single,
            DIVIDENDS.to_owned() => KvFileDbStoreType::Multi,
            UTXOS.to_owned() => KvFileDbStoreType::Single,
            CONSUMED_UTXOS.to_owned() => KvFileDbStoreType::SingleIntKey,
        ],
    }
}

/// Open database
#[inline]
pub fn open_db_ro(path: &Path) -> Result<BcDbRo, DbError> {
    BcDbRo::open_db_ro(path, &bc_db_schema())
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use durs_dbs_tools::kv_db::KvFileDbHandler;
    use tempfile::tempdir;

    #[inline]
    /// Open database in an arbitrary temporary directory given by OS
    /// and automatically cleaned when `Db` is dropped
    pub fn open_tmp_db() -> Result<KvFileDbHandler, DbError> {
        KvFileDbHandler::open_db(
            tempdir().map_err(DbError::FileSystemError)?.path(),
            &bc_db_schema(),
        )
    }
}

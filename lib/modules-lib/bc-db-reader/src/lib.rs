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

#![allow(clippy::large_enum_variant, clippy::ptr_arg)]
#![deny(
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
pub mod current_meta_datas;
pub mod indexes;
pub mod paging;
pub mod tools;
pub mod traits;

pub use durs_dbs_tools::kv_db::{
    from_db_value, KvFileDbRead as DbReadable, KvFileDbReader as Reader,
    KvFileDbRoHandler as BcDbRo, KvFileDbSchema, KvFileDbStoreType, KvFileDbValue as DbValue,
    Readable as DbReader,
};
pub use durs_dbs_tools::DbError;
#[cfg(feature = "mock")]
pub use traits::MockBcDbInReadTx;
pub use traits::{BcDbInReadTx, BcDbRead, BcDbWithReader};

use constants::*;
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

pub struct BcDbWithReaderStruct<'r, 'db: 'r, DB>
where
    DB: DbReadable,
{
    pub db: &'db DB,
    pub r: Reader<'r>,
}

pub type BcDbRoWithReader<'r, 'db> = BcDbWithReaderStruct<'r, 'db, BcDbRo>;

impl<'r, 'db: 'r, DB> BcDbWithReader for BcDbWithReaderStruct<'r, 'db, DB>
where
    DB: DbReadable,
{
    type DB = DB;
    type R = Reader<'r>;

    fn db(&self) -> &Self::DB {
        self.db
    }
    fn r(&self) -> &Self::R {
        &self.r
    }
}

impl<'r, 'db: 'r, DB> durs_common_tools::traits::NotMock for BcDbWithReaderStruct<'r, 'db, DB> where
    DB: DbReadable
{
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

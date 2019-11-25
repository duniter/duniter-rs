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
// ! Define read only trait

use crate::blocks::DbBlock;
use crate::{BcDbWithReaderStruct, DbReadable, DbReader};
use dubp_common_doc::{BlockNumber, Blockstamp};
use dup_crypto::keys::PubKey;
use durs_dbs_tools::DbError;
#[cfg(feature = "mock")]
use mockall::*;

pub trait BcDbRead<DB>
where
    DB: DbReadable,
{
    /// Read datas in Db
    fn r<D, F>(&self, f: F) -> Result<D, DbError>
    where
        DB: DbReadable,
        F: FnOnce(&BcDbWithReaderStruct<DB>) -> Result<D, DbError>;
}

impl<DB> BcDbRead<DB> for DB
where
    DB: DbReadable,
{
    fn r<D, F>(&self, f: F) -> Result<D, DbError>
    where
        DB: DbReadable,
        F: FnOnce(&BcDbWithReaderStruct<DB>) -> Result<D, DbError>,
    {
        self.read(|r| f(&BcDbWithReaderStruct { db: self, r }))
    }
}

pub trait BcDbWithReader {
    type DB: DbReadable;
    type R: DbReader;

    fn db(&self) -> &Self::DB;
    fn r(&self) -> &Self::R;
}

#[cfg(feature = "mock")]
impl<'a> BcDbWithReader for MockBcDbInReadTx {
    type DB = crate::BcDbRo;
    type R = durs_dbs_tools::kv_db::MockKvFileDbReader;

    fn db(&self) -> &Self::DB {
        unreachable!()
    }
    fn r(&self) -> &Self::R {
        unreachable!()
    }
}

#[cfg_attr(feature = "mock", automock)]
pub trait BcDbInReadTx: BcDbWithReader {
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError>;
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError>;
    fn get_db_block_in_local_blockchain(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError>;
    #[cfg(feature = "client-indexer")]
    fn get_db_blocks_in_local_blockchain(
        &self,
        numbers: Vec<BlockNumber>,
    ) -> Result<Vec<DbBlock>, DbError>;
    fn get_uid_from_pubkey(&self, pubkey: &PubKey) -> Result<Option<String>, DbError>;
}

impl<T> BcDbInReadTx for T
where
    T: BcDbWithReader + durs_common_tools::traits::NotMock,
{
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError> {
        crate::current_meta_datas::get_current_blockstamp(self)
    }
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError> {
        if let Some(current_blockstamp) = crate::current_meta_datas::get_current_blockstamp(self)? {
            crate::blocks::get_db_block_in_local_blockchain(self, current_blockstamp.id)
        } else {
            Ok(None)
        }
    }
    fn get_db_block_in_local_blockchain(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError> {
        crate::blocks::get_db_block_in_local_blockchain(self, block_number)
    }
    #[cfg(feature = "client-indexer")]
    fn get_db_blocks_in_local_blockchain(
        &self,
        numbers: Vec<BlockNumber>,
    ) -> Result<Vec<DbBlock>, DbError> {
        crate::blocks::get_blocks_in_local_blockchain_by_numbers(self, numbers)
    }
    fn get_uid_from_pubkey(&self, pubkey: &PubKey) -> Result<Option<String>, DbError> {
        crate::indexes::identities::get_uid_(self, pubkey)
    }
}

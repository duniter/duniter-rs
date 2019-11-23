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
use crate::{BcDbRo, DbReadable, Reader};
use dubp_common_doc::{BlockNumber, Blockstamp};
use durs_dbs_tools::DbError;

#[cfg(feature = "mock")]
use mockall::*;

#[cfg_attr(feature = "mock", automock)]
pub trait BcDbRoTrait {
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
}

impl BcDbRoTrait for BcDbRo {
    #[inline]
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError> {
        self.read(|r| crate::current_meta_datas::get_current_blockstamp_(self, r))
    }
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError> {
        self.read(|r| {
            if let Some(current_blockstamp) =
                crate::current_meta_datas::get_current_blockstamp_(self, r)?
            {
                crate::blocks::get_db_block_in_local_blockchain(self, r, current_blockstamp.id)
            } else {
                Ok(None)
            }
        })
    }
    #[inline]
    fn get_db_block_in_local_blockchain(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError> {
        self.read(|r| crate::blocks::get_db_block_in_local_blockchain(self, r, block_number))
    }
    #[cfg(feature = "client-indexer")]
    fn get_db_blocks_in_local_blockchain(
        &self,
        numbers: Vec<BlockNumber>,
    ) -> Result<Vec<DbBlock>, DbError> {
        self.read(|r| {
            numbers
                .into_iter()
                .filter_map(
                    |n| match crate::blocks::get_db_block_in_local_blockchain(self, r, n) {
                        Ok(Some(db_block)) => Some(Ok(db_block)),
                        Ok(None) => None,
                        Err(e) => Some(Err(e)),
                    },
                )
                .collect::<Result<Vec<DbBlock>, DbError>>()
        })
    }
}

pub struct BcDbRoWithReader<'r, 'db: 'r> {
    pub db: &'db BcDbRo,
    pub r: Reader<'r>,
}

impl<'r, 'db: 'r> BcDbRoTrait for BcDbRoWithReader<'r, 'db> {
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError> {
        crate::current_meta_datas::get_current_blockstamp_(self.db, self.r)
    }
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError> {
        unimplemented!()
    }
    fn get_db_block_in_local_blockchain(
        &self,
        _block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError> {
        unimplemented!()
    }
    fn get_db_blocks_in_local_blockchain(
        &self,
        _numbers: Vec<BlockNumber>,
    ) -> Result<Vec<DbBlock>, DbError> {
        unimplemented!()
    }
}

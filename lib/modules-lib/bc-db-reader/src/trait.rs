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
use crate::{BcDbRo, Reader};
use dubp_common_doc::{BlockNumber, Blockstamp};
use dup_crypto::keys::PubKey;
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
    fn get_uid_from_pubkey(&self, pubkey: &PubKey) -> Result<Option<String>, DbError>;
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
        if let Some(current_blockstamp) =
            crate::current_meta_datas::get_current_blockstamp_(self.db, self.r)?
        {
            crate::blocks::get_db_block_in_local_blockchain(self.db, self.r, current_blockstamp.id)
        } else {
            Ok(None)
        }
    }
    fn get_db_block_in_local_blockchain(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError> {
        crate::blocks::get_db_block_in_local_blockchain(self.db, self.r, block_number)
    }
    #[cfg(feature = "client-indexer")]
    fn get_db_blocks_in_local_blockchain(
        &self,
        numbers: Vec<BlockNumber>,
    ) -> Result<Vec<DbBlock>, DbError> {
        crate::blocks::get_blocks_in_local_blockchain_by_numbers(self.db, self.r, numbers)
    }
    fn get_uid_from_pubkey(&self, pubkey: &PubKey) -> Result<Option<String>, DbError> {
        crate::indexes::identities::get_uid_(self.db, self.r, pubkey)
    }
}

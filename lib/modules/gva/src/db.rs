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

//! Gva Module: database requests

pub use durs_bc_db_reader::DbError;

use dubp_common_doc::{BlockNumber, Blockstamp};
use durs_bc_db_reader::blocks::DbBlock;
use durs_bc_db_reader::{BcDbRo, DbReadable};
use std::ops::Range;

#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::*;

#[cfg_attr(test, automock)]
pub(crate) trait BcDbTrait {
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError>;
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError>;
    fn get_db_block_in_local_blockchain(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<DbBlock>, DbError>;
    fn get_db_blocks_in_local_blockchain(&self, range: Range<u32>)
        -> Result<Vec<DbBlock>, DbError>;
}

impl<'a> BcDbTrait for BcDbRo {
    #[inline]
    fn get_current_blockstamp(&self) -> Result<Option<Blockstamp>, DbError> {
        self.read(|r| durs_bc_db_reader::current_meta_datas::get_current_blockstamp_(self, r))
    }
    fn get_current_block(&self) -> Result<Option<DbBlock>, DbError> {
        self.read(|r| {
            if let Some(current_blockstamp) =
                durs_bc_db_reader::current_meta_datas::get_current_blockstamp_(self, r)?
            {
                durs_bc_db_reader::blocks::get_db_block_in_local_blockchain(
                    self,
                    r,
                    current_blockstamp.id,
                )
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
        self.read(|r| {
            durs_bc_db_reader::blocks::get_db_block_in_local_blockchain(self, r, block_number)
        })
    }
    fn get_db_blocks_in_local_blockchain(
        &self,
        range: Range<u32>,
    ) -> Result<Vec<DbBlock>, DbError> {
        self.read(|r| {
            range
                .filter_map(|n| {
                    match durs_bc_db_reader::blocks::get_db_block_in_local_blockchain(
                        self,
                        r,
                        BlockNumber(n),
                    ) {
                        Ok(Some(db_block)) => Some(Ok(db_block)),
                        Ok(None) => None,
                        Err(e) => Some(Err(e)),
                    }
                })
                .collect::<Result<Vec<DbBlock>, DbError>>()
        })
    }
}

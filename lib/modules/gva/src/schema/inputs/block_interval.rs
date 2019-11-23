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

// ! BlockInterval input methods

pub use crate::schema::BlockInterval;

use durs_bc_db_reader::{BcDbRoTrait, DbError};
use std::ops::RangeInclusive;

const DEFAULT_START: usize = 0;
const END_WHEN_EMPTY_BLOCKCHAIN: usize = 0;

impl BlockInterval {
    fn get_default_end<DB: BcDbRoTrait>(db: &DB) -> Result<usize, DbError> {
        if let Some(current_blockstamp) = db.get_current_blockstamp()? {
            Ok(current_blockstamp.id.0 as usize)
        } else {
            Ok(END_WHEN_EMPTY_BLOCKCHAIN)
        }
    }
    pub(crate) fn get_range<DB: BcDbRoTrait>(
        db: &DB,
        block_interval_opt: Option<BlockInterval>,
    ) -> Result<RangeInclusive<usize>, DbError> {
        if let Some(block_interval) = block_interval_opt {
            let start = if let Some(from) = block_interval.from {
                if from.is_negative() {
                    0
                } else {
                    from as usize
                }
            } else {
                DEFAULT_START
            };
            let mut end = if let Some(to) = block_interval.to {
                if to.is_negative() {
                    0
                } else {
                    to as usize
                }
            } else {
                Self::get_default_end(db)?
            };
            if start > end {
                end = start;
            }
            Ok(start..=end)
        } else {
            Ok(DEFAULT_START..=Self::get_default_end(db)?)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::db::BcDbRo;
    use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};

    #[test]
    fn test_block_interval_get_range_with_short_bc() -> Result<(), DbError> {
        let mut mock_db = BcDbRo::new();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(|| {
                Ok(Some(Blockstamp {
                    id: BlockNumber(42),
                    hash: BlockHash(dup_crypto::hashs::Hash::default()),
                }))
            });
        assert_eq! {
            0..=42,
            BlockInterval::get_range(&mock_db, None)?
        }
        Ok(())
    }

    #[test]
    fn test_block_interval_get_range_with_long_bc() -> Result<(), DbError> {
        let mut mock_db = BcDbRo::new();
        mock_db
            .expect_get_current_blockstamp()
            .times(2)
            .returning(|| {
                Ok(Some(Blockstamp {
                    id: BlockNumber(750),
                    hash: BlockHash(dup_crypto::hashs::Hash::default()),
                }))
            });

        assert_eq! {
            0..=750,
            BlockInterval::get_range(&mock_db, None)?
        }

        assert_eq! {
            500..=750,
            BlockInterval::get_range(&mock_db, Some(BlockInterval {
                from: Some(500),
                to: None,
            }))?
        }

        assert_eq! {
            500..=700,
            BlockInterval::get_range(&mock_db, Some(BlockInterval {
                from: Some(500),
                to: Some(700),
            }))?
        }

        Ok(())
    }
}

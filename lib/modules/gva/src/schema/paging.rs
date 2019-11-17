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

// ! Schema paging input

use super::Paging;
use durs_bc_db_reader::{BcDbRo, DbError, DbReader};
use std::ops::Range;

const DEFAULT_PAGE_NUMBER: i32 = 0;
const DEFAULT_PAGE_SIZE: i32 = 50;
const DEFAULT_FROM_BLOCK: i32 = 0;

const MAX_PAGE_NUMBER: i32 = std::i32::MAX;
const MAX_PAGE_SIZE: i32 = 500;
const MAX_FROM_BLOCK: i32 = std::i32::MAX;

/// Paging with all values filled in
pub struct FilledPaging {
    page_number: usize,
    page_size: usize,
    from_block: u32,
    to_block: u32,
}

#[inline]
fn i32_opt_to_positive_i32(int_opt: Option<i32>, default: i32) -> i32 {
    if let Some(int) = int_opt {
        if int < 0 {
            0
        } else {
            int
        }
    } else {
        default
    }
}

impl FilledPaging {
    pub fn new<R: DbReader>(
        db: &BcDbRo,
        r: &R,
        paging_opt: Option<Paging>,
    ) -> Result<Self, DbError> {
        if let Some(paging) = paging_opt {
            Ok(FilledPaging {
                page_number: std::cmp::min(
                    MAX_PAGE_NUMBER,
                    i32_opt_to_positive_i32(paging.page_number, DEFAULT_PAGE_NUMBER),
                ) as usize,
                page_size: std::cmp::min(
                    MAX_PAGE_SIZE,
                    i32_opt_to_positive_i32(paging.page_size, DEFAULT_PAGE_SIZE),
                ) as usize,
                from_block: std::cmp::min(
                    MAX_FROM_BLOCK,
                    i32_opt_to_positive_i32(paging.from_block, DEFAULT_FROM_BLOCK),
                ) as u32,
                to_block: if let Some(to_block) = paging.to_block {
                    if to_block < 0 {
                        0
                    } else {
                        to_block as u32
                    }
                } else {
                    Self::get_default_to_block(db, r)?
                },
            })
        } else {
            Ok(FilledPaging {
                page_number: DEFAULT_PAGE_NUMBER as usize,
                page_size: DEFAULT_PAGE_SIZE as usize,
                from_block: DEFAULT_FROM_BLOCK as u32,
                to_block: Self::get_default_to_block(db, r)?,
            })
        }
    }
    fn get_default_to_block<R: DbReader>(db: &BcDbRo, r: &R) -> Result<u32, DbError> {
        if let Some(current_blockstamp) =
            durs_bc_db_reader::current_meta_datas::get_current_blockstamp_(db, r)?
        {
            Ok(current_blockstamp.id.0)
        } else {
            Ok(0)
        }
    }
    pub fn get_range(&self) -> Range<u32> {
        Range {
            start: self.from_block + (self.page_number * self.page_size) as u32,
            end: std::cmp::min(
                self.to_block + 1,
                self.from_block + ((self.page_number + 1) * self.page_size) as u32,
            ),
        }
    }
}

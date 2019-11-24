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

use crate::constants::*;
use dubp_common_doc::BlockNumber;
use std::cmp::{max, min};
use std::ops::RangeInclusive;

pub struct FilledBlockInterval {
    from: usize,
    to: usize,
}

impl FilledBlockInterval {
    pub(crate) fn new(
        block_interval_opt: Option<BlockInterval>,
        current_block_number_opt: Option<BlockNumber>,
    ) -> Self {
        let current_block_number = current_block_number_opt.unwrap_or(BlockNumber(0)).0 as usize;

        if let Some(block_interval) = block_interval_opt {
            if let Some(from) = block_interval.from {
                if let Some(to) = block_interval.to {
                    Self::new_with_from_and_to(current_block_number, from, to)
                } else {
                    Self::new_with_from(current_block_number, from)
                }
            } else if let Some(to) = block_interval.to {
                Self::new_with_to(current_block_number, to)
            } else {
                Self::new_with_nothing(current_block_number)
            }
        } else {
            Self::new_with_nothing(current_block_number)
        }
    }
    fn new_with_from(current_block_number: usize, from: i32) -> Self {
        let mut from = max(from, 0) as usize;
        let to = min(current_block_number, from + BLOCK_INTERVAL_MAX_SIZE);

        from = min(from, to);

        FilledBlockInterval { from, to }
    }
    fn new_with_to(current_block_number: usize, to: i32) -> Self {
        let mut to = max(0, to) as usize;
        to = min(current_block_number, to);
        let mut from = if to >= BLOCK_INTERVAL_MAX_SIZE {
            to - BLOCK_INTERVAL_MAX_SIZE
        } else {
            BLOCK_INTERVAL_MIN_FROM
        };

        from = min(from, to);

        FilledBlockInterval { from, to }
    }
    fn new_with_from_and_to(current_block_number: usize, from: i32, to: i32) -> Self {
        let mut from = max(from, 0) as usize;
        let mut to = max(0, to) as usize;
        to = min(current_block_number, to);
        from = min(from, to);

        FilledBlockInterval { from, to }
    }
    fn new_with_nothing(current_block_number: usize) -> Self {
        let filled_to = current_block_number;
        let filled_from = if current_block_number >= BLOCK_INTERVAL_MAX_SIZE {
            current_block_number - BLOCK_INTERVAL_MAX_SIZE
        } else {
            0
        };
        FilledBlockInterval {
            from: filled_from,
            to: filled_to,
        }
    }
    #[inline]
    pub(crate) fn get_range(&self) -> RangeInclusive<usize> {
        self.from..=self.to
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dubp_common_doc::BlockNumber;

    #[test]
    fn test_block_interval_get_range_with_short_bc() {
        assert_eq! {
            0..=42,
            FilledBlockInterval::new(None, Some(BlockNumber(42))).get_range()
        }
    }

    #[test]
    fn test_block_interval_get_range_with_long_bc() {
        assert_eq! {
            0..=750,
            FilledBlockInterval::new(None, Some(BlockNumber(750))).get_range()
        }

        assert_eq! {
            500..=750,
            FilledBlockInterval::new(Some(BlockInterval {
                from: Some(500),
                to: None,
            }), Some(BlockNumber(750))).get_range()
        }

        assert_eq! {
            500..=700,
            FilledBlockInterval::new(Some(BlockInterval {
                from: Some(500),
                to: Some(700),
            }), Some(BlockNumber(750))).get_range()
        }
    }
}

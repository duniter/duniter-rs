//  Copyright (C) 2018  The Durs Project Developers.
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

pub mod identities;

use dubp_documents::BlockId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Pagination parameters
pub struct PagingFilter {
    /// Retrieve only the elements created after this block
    pub from: BlockId,
    /// Retrieve only the elements created before this block
    pub to: Option<BlockId>,
    /// Number of elements per page
    pub page_size: usize,
    /// Number of the page requested
    pub page_number: usize,
}

impl Default for PagingFilter {
    fn default() -> Self {
        PagingFilter {
            from: BlockId(0),
            to: None,
            page_size: *crate::constants::DEFAULT_PAGE_SIZE,
            page_number: 0,
        }
    }
}

impl PagingFilter {
    #[inline]
    /// Checks if a given element has been created in the requested period
    pub fn check_created_on(&self, created_on: BlockId, current_block_id: BlockId) -> bool {
        created_on >= self.from && created_on <= self.to.unwrap_or(current_block_id)
    }
    #[inline]
    /// Checks if a given element index is located in the current page
    pub fn is_in_page(&self, i: usize) -> bool {
        i >= self.page_size * self.page_number && i < self.page_size * (self.page_number + 1)
    }
}

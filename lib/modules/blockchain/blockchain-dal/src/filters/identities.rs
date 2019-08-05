//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! Define all filters applicable to identities

use super::PagingFilter;
use dup_crypto::keys::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Identities filter
pub struct IdentitiesFilter {
    /// Pagination parameters
    pub paging: PagingFilter,
    /// Filter identities by public key
    pub by_pubkey: Option<PubKey>,
}

impl Default for IdentitiesFilter {
    fn default() -> Self {
        IdentitiesFilter {
            paging: PagingFilter::default(),
            by_pubkey: None,
        }
    }
}

impl IdentitiesFilter {
    /// Create "by pubkey" filter
    pub fn by_pubkey(pubkey: PubKey) -> Self {
        IdentitiesFilter {
            paging: PagingFilter::default(),
            by_pubkey: Some(pubkey),
        }
    }
}

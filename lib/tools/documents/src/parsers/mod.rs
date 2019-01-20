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

/// Parsers for block
pub mod blocks;

/// Parsers for certifications
pub mod certifications;

/// Parsers for exclusions
pub mod excluded;

/// Parsers for identities
pub mod identities;

/// Parsers for memberships
pub mod memberships;

/// Parsers for revocations
pub mod revoked;

use crate::*;

#[derive(Debug, Fail)]
#[fail(display = "Fail to parse JSON Block : {:?}", cause)]
pub struct ParseBlockError {
    pub cause: String,
}

impl From<BaseConvertionError> for ParseBlockError {
    fn from(_: BaseConvertionError) -> ParseBlockError {
        ParseBlockError {
            cause: "base conversion error".to_owned(),
        }
    }
}

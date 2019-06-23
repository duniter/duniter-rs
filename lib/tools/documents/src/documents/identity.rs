//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Wrappers around Identity documents.

pub mod v10;

pub use v10::{IdentityDocumentV10, IdentityDocumentV10Stringified};

use crate::documents::*;

/// Identity document
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub enum IdentityDocument {
    /// Identity document V10
    V10(IdentityDocumentV10),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum IdentityDocumentStringified {
    V10(IdentityDocumentV10Stringified),
}

impl ToStringObject for IdentityDocument {
    type StringObject = IdentityDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            IdentityDocument::V10(idty) => {
                IdentityDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

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

//! Wrappers around Membership documents.

pub mod v10;

pub use v10::{MembershipDocumentV10, MembershipDocumentV10Stringified};

use crate::documents::*;

/// Wrap an Membership document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MembershipDocument {
    V10(MembershipDocumentV10),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MembershipDocumentStringified {
    V10(MembershipDocumentV10Stringified),
}

impl ToStringObject for MembershipDocument {
    type StringObject = MembershipDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            MembershipDocument::V10(idty) => {
                MembershipDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

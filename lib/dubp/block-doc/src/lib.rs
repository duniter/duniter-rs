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

//! Wrappers around Block document.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate log;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate serde_derive;

pub mod block;
pub mod parser;

use dubp_common_doc::traits::ToStringObject;
use dubp_user_docs::documents::{UserDocumentDUBP, UserDocumentDUBPStr};

pub use block::{
    BlockDocument, BlockDocumentStringified, BlockDocumentV10, BlockDocumentV10Stringified,
};

/// Document of DUBP (DUniter Blockhain Protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentDUBP {
    /// Block document.
    Block(Box<BlockDocument>),
    /// User document of DUBP (DUniter Blockhain Protocol)
    UserDocument(UserDocumentDUBP),
}

/// List of stringified document types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentDUBPStr {
    /// Block document (not yet implemented)
    Block(Box<BlockDocumentStringified>),
    /// Stringified user document.
    UserDocument(UserDocumentDUBPStr),
}

impl ToStringObject for DocumentDUBP {
    type StringObject = DocumentDUBPStr;

    fn to_string_object(&self) -> Self::StringObject {
        match *self {
            DocumentDUBP::Block(ref doc) => {
                DocumentDUBPStr::Block(Box::new(doc.to_string_object()))
            }
            DocumentDUBP::UserDocument(ref user_doc) => {
                DocumentDUBPStr::UserDocument(user_doc.to_string_object())
            }
        }
    }
}

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

//! Provide wrappers around Duniter blockchain documents for protocol version 10.

use super::{Document, ToProtocolDocument, ToSpecializedDocument, BlockchainProtocolVersion};

/// List of wrapped document types.
///
/// > TODO Add wrapped types in enum variants.
#[derive(Debug, Copy, Clone)]
pub enum DocumentType<'a> {
    /// Block document.
    Block(),

    /// Transaction document.
    Transaction(),

    /// Identity document.
    Identity(&'a IdentityDocument),

    /// Membership document.
    Membership(),

    /// Certification document.
    Certification(),

    /// Revocation document.
    Revocation(),
}

/// Wrap an identity document.
#[derive(Debug, Clone)]
pub struct IdentityDocument {
    /// Currency
    pub currency: String,
}

impl Document for IdentityDocument {
    fn version(&self) -> u16 {
        10
    }

    fn currency(&self) -> &str {
        &self.currency
    }
}

impl<'a> ToSpecializedDocument<'a, DocumentType<'a>> for IdentityDocument {
    fn specialize(&'a self) -> DocumentType<'a> {
        DocumentType::Identity(self)
    }
}

impl<'a> ToProtocolDocument<'a, BlockchainProtocolVersion<'a>> for IdentityDocument {
    fn associated_protocol(&'a self) -> BlockchainProtocolVersion<'a> {
        BlockchainProtocolVersion::V10(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_document_concrete_type() {
        let doc = IdentityDocument { currency: "test".to_string() };

        // we now consider `doc` to be only a Document, and try to get back to our specialized document

        if let BlockchainProtocolVersion::V10(to_spe_doc) = doc.associated_protocol() {
            if let DocumentType::Identity(_) = to_spe_doc.specialize() {

            } else {
                panic!("wront doc type");
            }
        } else {
            panic!("wrong version");
        }
    }
}

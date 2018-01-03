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


use super::{Document, ToProtocolDocument, ToSpecializedDocument, BlockchainProtocolVersion,
            DocumentType, TextDocument};
use Blockstamp;
use duniter_keys::ed25519;

/// Wrap an Identity document.
///
/// Must be created by parsing a text document (not done yet)
/// or using a builder (not done yet).
#[derive(Debug, Clone)]
pub struct IdentityDocument {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values
    /// must be extracted from it.
    content: String,

    /// Currency.
    currency: String,
    /// Unique ID
    unique_id: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Document issuer (there should be only one).
    issuers: Vec<ed25519::PublicKey>,
    /// Document signature (there should be only one).
    signatures: Vec<ed25519::Signature>,
}

impl Document<ed25519::PublicKey, ed25519::Signature> for IdentityDocument {
    fn version(&self) -> u16 {
        10
    }

    fn currency(&self) -> &str {
        &self.currency
    }

    fn issuers(&self) -> &Vec<ed25519::PublicKey> {
        &self.issuers
    }

    fn signatures(&self) -> &Vec<ed25519::Signature> {
        &self.signatures
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_text().as_bytes()
    }
}

impl<'a> TextDocument<'a> for IdentityDocument {
    fn as_text(&'a self) -> &'a str {
        &self.content
    }
}

impl<
    'a,
> ToProtocolDocument<'a, ed25519::PublicKey, ed25519::Signature, BlockchainProtocolVersion<'a>>
    for IdentityDocument {
    fn associated_protocol(&'a self) -> BlockchainProtocolVersion<'a> {
        BlockchainProtocolVersion::V10(self)
    }
}

impl<'a> ToSpecializedDocument<'a, ed25519::PublicKey, ed25519::Signature, DocumentType<'a>>
    for IdentityDocument {
    fn specialize(&'a self) -> DocumentType<'a> {
        DocumentType::Identity(self)
    }
}

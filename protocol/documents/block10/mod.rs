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

pub mod identity;

pub use self::identity::{IdentityDocument, IdentityDocumentBuilder};

use duniter_keys::{Signature, ed25519};
use documents::DocumentBuilder;
use super::Document;

/// List of wrapped document types.
///
/// > TODO Add wrapped types in enum variants.
#[derive(Debug, Clone)]
pub enum V10Document {
    /// Block document.
    Block(),

    /// Transaction document.
    Transaction(),

    /// Identity document.
    Identity(IdentityDocument),

    /// Membership document.
    Membership(),

    /// Certification document.
    Certification(),

    /// Revocation document.
    Revocation(),
}

/// Trait for a V10 document.
pub trait TextDocument: Document<ed25519::PublicKey, ed25519::Signature> {
    /// Return document as text.
    fn as_text(&self) -> &str;

    /// Return document as text with leading signatures.
    fn as_text_with_signatures(&self) -> String {
        let mut text = self.as_text().to_string();

        for sig in self.signatures() {
            text = format!("{}{}\n", text, sig.to_base64());
        }

        text
    }
}

/// Trait for a V10 document builder.
pub trait TextDocumentBuilder<D>
    : DocumentBuilder<ed25519::PublicKey, ed25519::PrivateKey, ed25519::Signature, D>
where
    D: TextDocument,
{
    /// Generate document text.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn generate_text(&self) -> String;

    /// Generate final document with signatures, and also return them in an array.
    ///
    /// Returns :
    /// - Text without signatures
    /// - Signatures
    fn build_signed_text(
        &self,
        private_keys: Vec<ed25519::PrivateKey>,
    ) -> (String, Vec<ed25519::Signature>) {
        use duniter_keys::PrivateKey;

        let text = self.generate_text();

        let signatures: Vec<_> = {
            let text_bytes = text.as_bytes();
            private_keys
                .iter()
                .map(|key| key.sign(text_bytes))
                .collect()
        };

        (text, signatures)
    }
}

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

pub use self::identity::IdentityDocument;

use duniter_keys::{ed25519, Signature};
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

/// Trait for a V10 document.
pub trait TextDocument<'a>
    : ToSpecializedDocument<'a, ed25519::PublicKey, ed25519::Signature, DocumentType<'a>>
    {
    /// Return document as text.
    fn as_text(&'a self) -> &'a str;

    /// Return document as text with leading signatures.
    fn as_text_with_signatures(&'a self) -> String {
        let mut text = self.as_text().to_string();

        for sig in self.signatures() {
            text = format!("{}{}\n", text, sig.to_base64());
        }

        text
    }
}

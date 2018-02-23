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

use duniter_crypto::keys::{PublicKey, ed25519};
use regex::Regex;

use Blockstamp;
use blockchain::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use blockchain::v10::documents::{StandardTextDocumentParser, TextDocument, TextDocumentBuilder,
                                 V10Document, V10DocumentParsingError};

/// Type of a Membership.
#[derive(Debug, Clone, Copy)]
pub enum MembershipType {
    /// The member wishes to opt-in.
    In(),
    /// The member wishes to opt-out.
    Out(),
}

/// Wrap an Membership document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone)]
pub struct MembershipDocument {
    /// Document as text.
    /// 
    /// Is used to check signatures, and other values mut be extracted from it.
    text: String,

    /// Name of the currency.
    currency: String,
    /// Document issuer (there should be only one).
    issuers: Vec<ed25519::PublicKey>,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Membership message.
    membership: MembershipType,
    /// Identity to use for this public key.
    identity_username: String,
    /// Identity document blockstamp.
    identity_blockstamp: Blockstamp,
    /// Document signature (there should be only one).
    signatures: Vec<ed25519::Signature>,
}

impl MembershipDocument {
    /// Membership message.
    pub fn membership(&self) -> MembershipType {
        self.membership
    }

    /// Identity to use for this public key.
    pub fn identity_username(&self) -> &str {
        &self.identity_username
    }
}

impl Document for MembershipDocument {
    type PublicKey = ed25519::PublicKey;
    type CurrencyType = str;

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

impl TextDocument for MembershipDocument {
    fn as_text(&self) -> &str {
        &self.text
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for MembershipDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(V10Document::Membership(self))
    }
}
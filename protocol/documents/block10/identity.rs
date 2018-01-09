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

use duniter_keys::ed25519;

use Blockstamp;
use documents::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use documents::block10::{TextDocument, TextDocumentBuilder, V10Document};

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
    text: String,

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

impl Document for IdentityDocument {
    type PublicKey = ed25519::PublicKey;

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

impl TextDocument for IdentityDocument {
    fn as_text(&self) -> &str {
        &self.text
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for IdentityDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(V10Document::Identity(self))
    }
}

/// Identity document builder.
#[derive(Debug, Copy, Clone)]
pub struct IdentityDocumentBuilder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Identity unique id.
    pub unique_id: &'a str,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Document/identity issuer.
    pub issuer: &'a ed25519::PublicKey,
}

impl<'a> DocumentBuilder for IdentityDocumentBuilder<'a> {
    type Document = IdentityDocument;
    type PrivateKey = ed25519::PrivateKey;

    fn build_with_signature(self, signatures: Vec<ed25519::Signature>) -> IdentityDocument {
        IdentityDocument {
            text: self.generate_text(),
            currency: self.currency.to_string(),
            unique_id: self.unique_id.to_string(),
            blockstamp: *self.blockstamp,
            issuers: vec![*self.issuer],
            signatures,
        }
    }

    fn build_and_sign(self, private_keys: Vec<ed25519::PrivateKey>) -> IdentityDocument {
        let (text, signatures) = self.build_signed_text(private_keys);

        IdentityDocument {
            text: text,
            currency: self.currency.to_string(),
            unique_id: self.unique_id.to_string(),
            blockstamp: *self.blockstamp,
            issuers: vec![*self.issuer],
            signatures,
        }
    }
}

impl<'a> TextDocumentBuilder<IdentityDocument> for IdentityDocumentBuilder<'a> {
    fn generate_text(&self) -> String {
        format!(
            "Version: 10
Type: Identity
Currency: {currency}
Issuer: {issuer}
UniqueID: {unique_id}
Timestamp: {blockstamp}
",
            currency = self.currency,
            issuer = self.issuer,
            unique_id = self.unique_id,
            blockstamp = self.blockstamp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_keys::{PrivateKey, PublicKey, Signature};
    use documents::VerificationResult;

    #[test]
    fn generate_real_document() {
        let pubkey = ed25519::PublicKey::from_base58(
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV",
        ).unwrap();

        let prikey = ed25519::PrivateKey::from_base58(
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5G\
             iERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
        ).unwrap();

        let sig = ed25519::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGM\
             MmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap();

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        ).unwrap();

        {
            let doc = IdentityDocumentBuilder {
                currency: "duniter_unit_test_currency",
                unique_id: "tic",
                blockstamp: &block,
                issuer: &pubkey,
            }.build_with_signature(vec![sig]);

            assert_eq!(doc.verify_signatures(), VerificationResult::Valid());
        }

        {
            let doc = IdentityDocumentBuilder {
                currency: "duniter_unit_test_currency",
                unique_id: "tic",
                blockstamp: &block,
                issuer: &pubkey,
            }.build_and_sign(vec![prikey]);

            assert_eq!(doc.verify_signatures(), VerificationResult::Valid());
        }
    }
}

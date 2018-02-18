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

use duniter_keys::{PublicKey, ed25519};
use regex::Regex;

use Blockstamp;
use documents::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use documents::block10::{StandardTextDocumentParser, TextDocument, TextDocumentBuilder,
                         V10Document, V10DocumentParsingError};

lazy_static! {
    static ref IDENTITY_REGEX: Regex = Regex::new(
        "^Issuer: (?P<issuer>[1-9A-Za-z][^OIl]{43,44})\nUniqueID: (?P<uid>[[:alnum:]_-]+)\nTimestamp: (?P<blockstamp>[0-9]+-[0-9A-F]{64})\n$"
    ).unwrap();
}

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

impl<'a> IdentityDocumentBuilder<'a> {
    fn build_with_text_and_sigs(
        self,
        text: String,
        signatures: Vec<ed25519::Signature>,
    ) -> IdentityDocument {
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

impl<'a> DocumentBuilder for IdentityDocumentBuilder<'a> {
    type Document = IdentityDocument;
    type PrivateKey = ed25519::PrivateKey;

    fn build_with_signature(self, signatures: Vec<ed25519::Signature>) -> IdentityDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(self, private_keys: Vec<ed25519::PrivateKey>) -> IdentityDocument {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for IdentityDocumentBuilder<'a> {
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

/// Identity document parser
#[derive(Debug, Clone, Copy)]
pub struct IdentityDocumentParser;

impl StandardTextDocumentParser for IdentityDocumentParser {
    fn parse_standard(
        doc: &str,
        body: &str,
        currency: &str,
        signatures: Vec<ed25519::Signature>,
    ) -> Result<V10Document, V10DocumentParsingError> {
        if let Some(caps) = IDENTITY_REGEX.captures(body) {
            let issuer = &caps["issuer"];
            let uid = &caps["uid"];
            let blockstamp = &caps["blockstamp"];

            // Regex match so should not fail.
            // TODO : Test it anyway
            let issuer = ed25519::PublicKey::from_base58(issuer).unwrap();
            let blockstamp = Blockstamp::from_string(blockstamp).unwrap();

            Ok(V10Document::Identity(IdentityDocument {
                text: doc.to_owned(),
                currency: currency.to_owned(),
                unique_id: uid.to_owned(),
                blockstamp: blockstamp,
                issuers: vec![issuer],
                signatures,
            }))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Identity".to_string(),
            ))
        }
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

    #[test]
    fn identity_standard_regex() {
        assert!(IDENTITY_REGEX.is_match(
            "Issuer: DKpQPUL4ckzXYdnDRvCRKAm1gNvSdmAXnTrJZ7LvM5Qo
UniqueID: toc
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
"
        ));
    }

    #[test]
    fn parse_identity_document() {
        let doc = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DKpQPUL4ckzXYdnDRvCRKAm1gNvSdmAXnTrJZ7LvM5Qo
UniqueID: toc
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
lcekuS0eP2dpFL99imJcwvDAwx49diiDMkG8Lj7FLkC/6IJ0tgNjUzCIZgMGi7bL5tODRiWi9B49UMXb8b3MAw==";

        let body = "Issuer: DKpQPUL4ckzXYdnDRvCRKAm1gNvSdmAXnTrJZ7LvM5Qo
UniqueID: toc
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let currency = "duniter_unit_test_currency";

        let signatures = vec![Signature::from_base64(
"lcekuS0eP2dpFL99imJcwvDAwx49diiDMkG8Lj7FLkC/6IJ0tgNjUzCIZgMGi7bL5tODRiWi9B49UMXb8b3MAw=="
        ).unwrap(),];

        let _ = IdentityDocumentParser::parse_standard(doc, body, currency, signatures).unwrap();
    }
}

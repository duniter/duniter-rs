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

extern crate serde;

use self::serde::ser::{Serialize, Serializer};
use duniter_crypto::keys::*;
use regex::Regex;

use blockchain::v10::documents::*;
use blockchain::{BlockchainProtocol, Document, DocumentBuilder, IntoSpecializedDocument};
use Blockstamp;

lazy_static! {
    static ref IDENTITY_REGEX: Regex = Regex::new(
        "^Issuer: (?P<issuer>[1-9A-Za-z][^OIl]{43,44})\nUniqueID: (?P<uid>[[:alnum:]_-]+)\nTimestamp: (?P<blockstamp>[0-9]+-[0-9A-F]{64})\n$"
    ).unwrap();
}

/// Wrap an Identity document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IdentityDocument {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values
    /// must be extracted from it.
    text: String,

    /// Currency.
    currency: String,
    /// Unique ID
    username: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Document issuer (there should be only one).
    issuers: Vec<PubKey>,
    /// Document signature (there should be only one).
    signatures: Vec<Sig>,
}

impl IdentityDocument {
    /// Unique ID
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Document for IdentityDocument {
    type PublicKey = PubKey;
    type CurrencyType = str;

    fn version(&self) -> u16 {
        10
    }

    fn currency(&self) -> &str {
        &self.currency
    }

    fn blockstamp(&self) -> Blockstamp {
        self.blockstamp
    }

    fn issuers(&self) -> &Vec<PubKey> {
        &self.issuers
    }

    fn signatures(&self) -> &Vec<Sig> {
        &self.signatures
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_text().as_bytes()
    }
}

impl CompactTextDocument for IdentityDocument {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{signature}:{blockstamp}:{username}",
            issuer = self.issuers[0],
            signature = self.signatures[0],
            blockstamp = self.blockstamp,
            username = self.username,
        )
    }
}

impl TextDocument for IdentityDocument {
    type CompactTextDocument_ = IdentityDocument;

    fn as_text(&self) -> &str {
        &self.text
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        self.clone()
    }
}

impl Serialize for IdentityDocument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.generate_compact_text())
    }
}

impl IntoSpecializedDocument<BlockchainProtocol> for IdentityDocument {
    fn into_specialized(self) -> BlockchainProtocol {
        BlockchainProtocol::V10(Box::new(V10Document::Identity(self)))
    }
}

/// Identity document builder.
#[derive(Debug, Copy, Clone)]
pub struct IdentityDocumentBuilder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Identity unique id.
    pub username: &'a str,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Document/identity issuer.
    pub issuer: &'a PubKey,
}

impl<'a> IdentityDocumentBuilder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> IdentityDocument {
        IdentityDocument {
            text,
            currency: self.currency.to_string(),
            username: self.username.to_string(),
            blockstamp: *self.blockstamp,
            issuers: vec![*self.issuer],
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for IdentityDocumentBuilder<'a> {
    type Document = IdentityDocument;
    type PrivateKey = PrivKey;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> IdentityDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<PrivKey>) -> IdentityDocument {
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
UniqueID: {username}
Timestamp: {blockstamp}
",
            currency = self.currency,
            issuer = self.issuer,
            username = self.username,
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
        signatures: Vec<Sig>,
    ) -> Result<V10Document, V10DocumentParsingError> {
        if let Some(caps) = IDENTITY_REGEX.captures(body) {
            let issuer = &caps["issuer"];
            let uid = &caps["uid"];
            let blockstamp = &caps["blockstamp"];

            // Regex match so should not fail.
            // TODO : Test it anyway
            let issuer = PubKey::Ed25519(ed25519::PublicKey::from_base58(issuer).unwrap());
            let blockstamp = Blockstamp::from_string(blockstamp).unwrap();

            Ok(V10Document::Identity(IdentityDocument {
                text: doc.to_owned(),
                currency: currency.to_owned(),
                username: uid.to_owned(),
                blockstamp,
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
    use blockchain::{Document, VerificationResult};
    use duniter_crypto::keys::{PrivateKey, PublicKey, Signature};

    #[test]
    fn generate_real_document() {
        let pubkey = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let prikey = PrivKey::Ed25519(
            ed25519::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5G\
                 iERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
            ).unwrap(),
        );

        let sig = Sig::Ed25519(
            ed25519::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGM\
                 MmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
            ).unwrap(),
        );

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        ).unwrap();

        let builder = IdentityDocumentBuilder {
            currency: "duniter_unit_test_currency",
            username: "tic",
            blockstamp: &block,
            issuer: &pubkey,
        };

        assert_eq!(
            builder.build_with_signature(vec![sig]).verify_signatures(),
            VerificationResult::Valid()
        );
        assert_eq!(
            builder.build_and_sign(vec![prikey]).verify_signatures(),
            VerificationResult::Valid()
        );
    }

    #[test]
    fn identity_standard_regex() {
        assert!(IDENTITY_REGEX.is_match(
            "Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
"
        ));
    }

    #[test]
    fn parse_identity_document() {
        let doc = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let body = "Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let currency = "duniter_unit_test_currency";

        let signatures = vec![Sig::Ed25519(ed25519::Signature::from_base64(
"1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg=="
        ).unwrap())];

        let doc = IdentityDocumentParser::parse_standard(doc, body, currency, signatures).unwrap();
        if let V10Document::Identity(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
    }
}

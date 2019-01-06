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

use pest::Parser;

use crate::documents::*;
use crate::text_document_traits::*;
use crate::Blockstamp;

/// Wrap an Identity document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub struct IdentityDocument {
    /// Document as text.
    ///
    /// Is used to check signatures, and other values
    /// must be extracted from it.
    text: Option<String>,

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

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
/// identity document for jsonification
pub struct IdentityStringDocument {
    /// Currency.
    currency: String,
    /// Unique ID
    username: String,
    /// Blockstamp
    blockstamp: String,
    /// Document issuer
    issuer: String,
    /// Document signature
    signature: String,
}

impl ToStringObject for IdentityDocument {
    type StringObject = IdentityStringDocument;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> IdentityStringDocument {
        IdentityStringDocument {
            currency: self.currency.clone(),
            username: self.username.clone(),
            blockstamp: format!("{}", self.blockstamp),
            issuer: format!("{}", self.issuers[0]),
            signature: format!("{}", self.signatures[0]),
        }
    }
}

impl IdentityDocument {
    /// Unique ID
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Lightens the membership (for example to store it while minimizing the space required)
    pub fn reduce(&mut self) {
        self.text = None;
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
        self.as_text_without_signature().as_bytes()
    }
}

/// CompactIdentityDocument
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub struct CompactIdentityDocument {
    /// Unique ID
    username: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Document issuer
    pubkey: PubKey,
    /// Document signature
    signature: Sig,
}

impl CompactTextDocument for CompactIdentityDocument {
    fn as_compact_text(&self) -> String {
        format!(
            "{issuer}:{signature}:{blockstamp}:{username}",
            issuer = self.pubkey,
            signature = self.signature,
            blockstamp = self.blockstamp,
            username = self.username,
        )
    }
}

impl TextDocument for IdentityDocument {
    type CompactTextDocument_ = CompactIdentityDocument;

    fn as_text(&self) -> &str {
        if let Some(ref text) = self.text {
            text
        } else {
            panic!("Try to get text of reduce identity !")
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        CompactIdentityDocument {
            username: self.username.clone(),
            blockstamp: self.blockstamp,
            pubkey: self.issuers[0],
            signature: self.signatures[0],
        }
    }
}

impl IntoSpecializedDocument<DUBPDocument> for IdentityDocument {
    fn into_specialized(self) -> DUBPDocument {
        DUBPDocument::Identity(self)
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
            text: Some(text),
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

impl TextDocumentParser<Rule> for IdentityDocumentParser {
    type DocumentType = IdentityDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        match DocumentsParser::parse(Rule::idty, doc) {
            Ok(mut doc_pairs) => {
                let idty_pair = doc_pairs.next().unwrap(); // get and unwrap the `idty` rule; never fails
                let idty_vx_pair = idty_pair.into_inner().next().unwrap(); // get and unwrap the `idty_vx` rule; never fails

                match idty_vx_pair.as_rule() {
                    Rule::idty_v10 => Ok(IdentityDocumentParser::from_pest_pair(idty_vx_pair)),
                    _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                        "{:#?}",
                        idty_vx_pair.as_rule()
                    ))),
                }
            }
            Err(pest_error) => Err(TextDocumentParseError::PestError(format!("{}", pest_error))),
        }
    }
    fn from_pest_pair(pair: Pair<Rule>) -> Self::DocumentType {
        let doc = pair.as_str();
        let mut currency = "";
        let mut pubkey_str = "";
        let mut uid = "";
        let mut blockstamp = Blockstamp::default();
        let mut sig_str = "";
        for field in pair.into_inner() {
            match field.as_rule() {
                Rule::currency => currency = field.as_str(),
                Rule::pubkey => pubkey_str = field.as_str(),
                Rule::uid => uid = field.as_str(),
                Rule::blockstamp => {
                    let mut inner_rules = field.into_inner(); // { integer ~ "-" ~ hash }

                    let block_id: &str = inner_rules.next().unwrap().as_str();
                    let block_hash: &str = inner_rules.next().unwrap().as_str();
                    blockstamp = Blockstamp {
                        id: BlockId(block_id.parse().unwrap()), // Grammar ensures that we have a digits string.
                        hash: BlockHash(Hash::from_hex(block_hash).unwrap()), // Grammar ensures that we have an hexadecimal string.
                    };
                }
                Rule::ed25519_sig => sig_str = field.as_str(),
                Rule::EOI => (),
                _ => panic!("unexpected rule"), // Grammar ensures that we never reach this line
            }
        }
        IdentityDocument {
            text: Some(doc.to_owned()),
            currency: currency.to_owned(),
            username: uid.to_owned(),
            blockstamp,
            issuers: vec![PubKey::Ed25519(
                ed25519::PublicKey::from_base58(pubkey_str).unwrap(),
            )], // Grammar ensures that we have a base58 string.
            signatures: vec![Sig::Ed25519(
                ed25519::Signature::from_base64(sig_str).unwrap(),
            )], // Grammar ensures that we have a base64 string.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, VerificationResult};
    use dup_crypto::keys::{PrivateKey, PublicKey, Signature};

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
            )
            .unwrap(),
        );

        let sig = Sig::Ed25519(
            ed25519::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGM\
                 MmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
            )
            .unwrap(),
        );

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .unwrap();

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
    fn parse_identity_document() {
        let doc = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==";

        let doc = IdentityDocumentParser::parse(doc).expect("Fail to parse idty doc !");
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }
}

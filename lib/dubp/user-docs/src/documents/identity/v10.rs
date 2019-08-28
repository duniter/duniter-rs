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

//! Wrappers around Identity documents V10.

use durs_common_tools::fatal_error;

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::TextDocumentParseError;
use dubp_common_doc::traits::text::*;
use dubp_common_doc::traits::{Document, DocumentBuilder, ToStringObject};
use dubp_common_doc::{BlockHash, BlockNumber};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;

/// Wrap an Identity document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub struct IdentityDocumentV10 {
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
pub struct IdentityDocumentV10Stringified {
    /// Currency.
    pub currency: String,
    /// Unique ID
    pub username: String,
    /// Blockstamp
    pub blockstamp: String,
    /// Document issuer
    pub issuer: String,
    /// Document signature
    pub signature: String,
}

impl ToStringObject for IdentityDocumentV10 {
    type StringObject = IdentityDocumentV10Stringified;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> IdentityDocumentV10Stringified {
        IdentityDocumentV10Stringified {
            currency: self.currency.clone(),
            username: self.username.clone(),
            blockstamp: format!("{}", self.blockstamp),
            issuer: format!("{}", self.issuers[0]),
            signature: format!("{}", self.signatures[0]),
        }
    }
}

impl IdentityDocumentV10 {
    /// Unique ID
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Lightens the membership (for example to store it while minimizing the space required)
    pub fn reduce(&mut self) {
        self.text = None;
    }
    /// From pest parser pair
    pub fn from_pest_pair(pair: Pair<Rule>) -> Result<IdentityDocumentV10, TextDocumentParseError> {
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
                        id: BlockNumber(block_id.parse().unwrap()), // Grammar ensures that we have a digits string.
                        hash: BlockHash(Hash::from_hex(block_hash).unwrap()), // Grammar ensures that we have an hexadecimal string.
                    };
                }
                Rule::ed25519_sig => sig_str = field.as_str(),
                Rule::EOI => (),
                _ => fatal_error!("unexpected rule"), // Grammar ensures that we never reach this line
            }
        }

        Ok(IdentityDocumentV10 {
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
        })
    }
}

impl Document for IdentityDocumentV10 {
    type PublicKey = PubKey;

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

/// CompactIdentityDocumentV10
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub struct CompactIdentityDocumentV10 {
    /// Unique ID
    username: String,
    /// Blockstamp
    blockstamp: Blockstamp,
    /// Document issuer
    pubkey: PubKey,
    /// Document signature
    signature: Sig,
}

impl CompactTextDocument for CompactIdentityDocumentV10 {
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

impl TextDocument for IdentityDocumentV10 {
    type CompactTextDocument_ = CompactIdentityDocumentV10;

    fn as_text(&self) -> &str {
        if let Some(ref text) = self.text {
            text
        } else {
            fatal_error!("Try to get text of reduce identity !")
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        CompactIdentityDocumentV10 {
            username: self.username.clone(),
            blockstamp: self.blockstamp,
            pubkey: self.issuers[0],
            signature: self.signatures[0],
        }
    }
}

/// Identity document builder.
#[derive(Debug, Copy, Clone)]
pub struct IdentityDocumentV10Builder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Identity unique id.
    pub username: &'a str,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Document/identity issuer.
    pub issuer: &'a PubKey,
}

impl<'a> IdentityDocumentV10Builder<'a> {
    fn build_with_text_and_sigs(self, text: String, signatures: Vec<Sig>) -> IdentityDocumentV10 {
        IdentityDocumentV10 {
            text: Some(text),
            currency: self.currency.to_string(),
            username: self.username.to_string(),
            blockstamp: *self.blockstamp,
            issuers: vec![*self.issuer],
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for IdentityDocumentV10Builder<'a> {
    type Document = IdentityDocumentV10;
    type Signator = SignatorEnum;

    fn build_with_signature(&self, signatures: Vec<Sig>) -> IdentityDocumentV10 {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<SignatorEnum>) -> IdentityDocumentV10 {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for IdentityDocumentV10Builder<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use dubp_common_doc::traits::Document;
    use dup_crypto::keys::Signature;

    #[test]
    fn generate_real_document() {
        let keypair = ed25519::KeyPairFromSeed32Generator::generate(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV").unwrap(),
        );
        let pubkey = PubKey::Ed25519(keypair.public_key());
        let signator =
            SignatorEnum::Ed25519(keypair.generate_signator().expect("fail to gen signator"));

        let sig = Sig::Ed25519(
            ed25519::Signature::from_base64(
                "mmFepRsiOjILKnCvEvN3IZScLOfg8+e0JPAl5VkiuTLZRGJKgKhPy8nQlCKbeg0jefQm/2HJ78e/Sj+NMqYLCw==",
            )
            .unwrap(),
        );

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        )
        .unwrap();

        let builder = IdentityDocumentV10Builder {
            currency: "duniter_unit_test_currency",
            username: "tic",
            blockstamp: &block,
            issuer: &pubkey,
        };

        /*println!(
            "Signatures = {:?}",
            builder
                .build_and_sign(vec![SignatorEnum::Ed25519(
                    keypair.generate_signator().expect("fail to gen signator")
                )])
                .signatures()
        );*/

        assert!(builder
            .build_with_signature(vec![sig])
            .verify_signatures()
            .is_ok());
        assert!(builder
            .build_and_sign(vec![signator])
            .verify_signatures()
            .is_ok());
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
        assert!(doc.verify_signatures().is_ok())
    }
}

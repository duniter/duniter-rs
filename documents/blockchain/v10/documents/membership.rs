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

lazy_static! {
    static ref MEMBERSHIP_REGEX: Regex = Regex::new(
        "^Issuer: (?P<issuer>[1-9A-Za-z][^OIl]{43,44})\n\
         Block: (?P<blockstamp>[0-9]+-[0-9A-F]{64})\n\
         Membership: (?P<membership>(IN|OUT))\n\
         UserID: (?P<ity_user>[[:alnum:]_-]+)\n\
         CertTS: (?P<ity_block>[0-9]+-[0-9A-F]{64})\n$"
    ).unwrap();
}

/// Type of a Membership.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum MembershipType {
    /// The member wishes to opt-in.
    In(),
    /// The member wishes to opt-out.
    Out(),
}

/// Wrap an Membership document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Hash)]
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
        BlockchainProtocol::V10(Box::new(V10Document::Membership(self)))
    }
}

/// Membership document builder.
#[derive(Debug, Copy, Clone)]
pub struct MembershipDocumentBuilder<'a> {
    /// Document currency.
    pub currency: &'a str,
    /// Document/identity issuer.
    pub issuer: &'a ed25519::PublicKey,
    /// Reference blockstamp.
    pub blockstamp: &'a Blockstamp,
    /// Membership message.
    pub membership: MembershipType,
    /// Identity username.
    pub identity_username: &'a str,
    /// Identity document blockstamp.
    pub identity_blockstamp: &'a Blockstamp,
}

impl<'a> MembershipDocumentBuilder<'a> {
    fn build_with_text_and_sigs(
        self,
        text: String,
        signatures: Vec<ed25519::Signature>,
    ) -> MembershipDocument {
        MembershipDocument {
            text,
            currency: self.currency.to_string(),
            issuers: vec![*self.issuer],
            blockstamp: *self.blockstamp,
            membership: self.membership,
            identity_username: self.identity_username.to_string(),
            identity_blockstamp: *self.identity_blockstamp,
            signatures,
        }
    }
}

impl<'a> DocumentBuilder for MembershipDocumentBuilder<'a> {
    type Document = MembershipDocument;
    type PrivateKey = ed25519::PrivateKey;

    fn build_with_signature(&self, signatures: Vec<ed25519::Signature>) -> MembershipDocument {
        self.build_with_text_and_sigs(self.generate_text(), signatures)
    }

    fn build_and_sign(&self, private_keys: Vec<ed25519::PrivateKey>) -> MembershipDocument {
        let (text, signatures) = self.build_signed_text(private_keys);
        self.build_with_text_and_sigs(text, signatures)
    }
}

impl<'a> TextDocumentBuilder for MembershipDocumentBuilder<'a> {
    fn generate_text(&self) -> String {
        format!(
            "Version: 10
Type: Membership
Currency: {currency}
Issuer: {issuer}
Block: {blockstamp}
Membership: {membership}
UserID: {username}
CertTS: {ity_blockstamp}
",
            currency = self.currency,
            issuer = self.issuer,
            blockstamp = self.blockstamp,
            membership = match self.membership {
                MembershipType::In() => "IN",
                MembershipType::Out() => "OUT",
            },
            username = self.identity_username,
            ity_blockstamp = self.identity_blockstamp,
        )
    }

    fn generate_compact_text(&self, signatures: Vec<ed25519::Signature>) -> String {
        format!(
            "{issuer}:{signature}:{blockstamp}:{idty_blockstamp}:{username}",
            issuer = self.issuer,
            signature = signatures[0],
            blockstamp = self.blockstamp,
            idty_blockstamp = self.identity_blockstamp,
            username = self.identity_username,
        )
    }
}

/// Membership document parser
#[derive(Debug, Clone, Copy)]
pub struct MembershipDocumentParser;

impl StandardTextDocumentParser for MembershipDocumentParser {
    fn parse_standard(
        doc: &str,
        body: &str,
        currency: &str,
        signatures: Vec<ed25519::Signature>,
    ) -> Result<V10Document, V10DocumentParsingError> {
        if let Some(caps) = MEMBERSHIP_REGEX.captures(body) {
            let issuer = &caps["issuer"];

            let blockstamp = &caps["blockstamp"];
            let membership = &caps["membership"];
            let username = &caps["ity_user"];
            let ity_block = &caps["ity_block"];

            // Regex match so should not fail.
            // TODO : Test it anyway
            let issuer = ed25519::PublicKey::from_base58(issuer).unwrap();
            let blockstamp = Blockstamp::from_string(blockstamp).unwrap();
            let membership = match membership {
                "IN" => MembershipType::In(),
                "OUT" => MembershipType::Out(),
                _ => panic!("Invalid membership type {}", membership),
            };

            let ity_block = Blockstamp::from_string(ity_block).unwrap();

            Ok(V10Document::Membership(MembershipDocument {
                text: doc.to_owned(),
                issuers: vec![issuer],
                currency: currency.to_owned(),
                blockstamp,
                membership,
                identity_username: username.to_owned(),
                identity_blockstamp: ity_block,
                signatures,
            }))
        } else {
            Err(V10DocumentParsingError::InvalidInnerFormat(
                "Membership".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_crypto::keys::{PrivateKey, PublicKey, Signature};
    use blockchain::VerificationResult;

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
            "s2hUbokkibTAWGEwErw6hyXSWlWFQ2UWs2PWx8d/kkEl\
             AyuuWaQq4Tsonuweh1xn4AC1TVWt4yMR3WrDdkhnAw==",
        ).unwrap();

        let block = Blockstamp::from_string(
            "0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        ).unwrap();

        let builder = MembershipDocumentBuilder {
            currency: "duniter_unit_test_currency",
            issuer: &pubkey,
            blockstamp: &block,
            membership: MembershipType::In(),
            identity_username: "tic",
            identity_blockstamp: &block,
        };

        assert_eq!(
            builder.build_with_signature(vec![sig]).verify_signatures(),
            VerificationResult::Valid()
        );
        assert_eq!(
            builder.build_and_sign(vec![prikey]).verify_signatures(),
            VerificationResult::Valid()
        );
        assert_eq!(
            builder.generate_compact_text(vec![sig]),
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV:\
            s2hUbokkibTAWGEwErw6hyXSWlWFQ2UWs2PWx8d/kkElAyuuWaQq4Tsonuweh1xn4AC1TVWt4yMR3WrDdkhnAw==:\
            0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855:\
            0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855:\
            tic"
        );
    }

    #[test]
    fn membership_standard_regex() {
        assert!(MEMBERSHIP_REGEX.is_match(
            "Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
Block: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Membership: IN
UserID: tic
CertTS: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
"
        ));
    }

    #[test]
    fn membership_identity_document() {
        let doc = "Version: 10
Type: Membership
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
Block: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Membership: IN
UserID: tic
CertTS: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let body = "Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
Block: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Membership: IN
UserID: tic
CertTS: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let currency = "duniter_unit_test_currency";

        let signatures = vec![Signature::from_base64(
"s2hUbokkibTAWGEwErw6hyXSWlWFQ2UWs2PWx8d/kkElAyuuWaQq4Tsonuweh1xn4AC1TVWt4yMR3WrDdkhnAw=="
        ).unwrap(),];

        let doc =
            MembershipDocumentParser::parse_standard(doc, body, currency, signatures).unwrap();
        if let V10Document::Membership(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
    }
}

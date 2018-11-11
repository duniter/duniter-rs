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

pub mod block;
pub mod certification;
pub mod identity;
pub mod membership;
pub mod revocation;
pub mod transaction;

use dup_crypto::keys::PrivateKey;
use pest::Parser;

pub use v10::block::BlockDocument;
use v10::certification::*;
use v10::identity::*;
use v10::membership::*;
use v10::revocation::*;
use v10::transaction::*;
use *;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Contains a document in full or compact format
pub enum TextDocumentFormat<D: TextDocument> {
    /// Complete format (Allows to check the validity of the signature)
    Complete(D),
    /// Format present in the blocks (does not always allow to verify the signature)
    Compact(D::CompactTextDocument_),
}

impl<D: TextDocument> TextDocumentFormat<D> {
    /// To compact document
    pub fn to_compact_document(&self) -> D::CompactTextDocument_ {
        match *self {
            TextDocumentFormat::Complete(ref doc) => doc.to_compact_document(),
            TextDocumentFormat::Compact(ref compact_doc) => (*compact_doc).clone(),
        }
    }
}

/// List of wrapped document types.
#[derive(Debug, Clone)]
pub enum V10Document {
    /// Block document.
    Block(Box<BlockDocument>),

    /// Transaction document.
    Transaction(Box<TransactionDocument>),

    /// Identity document.
    Identity(IdentityDocument),

    /// Membership document.
    Membership(MembershipDocument),

    /// Certification document.
    Certification(Box<CertificationDocument>),

    /// Revocation document.
    Revocation(Box<RevocationDocument>),
}

impl TextDocumentParser for V10Document {
    type DocumentType = V10Document;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        match DocumentsParser::parse(Rule::document_v10, doc) {
            Ok(mut document_v10_pairs) => Ok(V10Document::from_pest_pair(
                document_v10_pairs.next().unwrap(),
            )), // get and unwrap the `document_v10` rule; never fails
            Err(pest_error) => Err(TextDocumentParseError::PestError(format!("{}", pest_error))),
        }
    }
    fn from_pest_pair(pair: Pair<Rule>) -> Self::DocumentType {
        let doc_type_v10_pair = pair.into_inner().next().unwrap(); // get and unwrap the `{DOC_TYPE}_v10` rule; never fails

        match doc_type_v10_pair.as_rule() {
            Rule::idty_v10 => V10Document::Identity(
                identity::IdentityDocumentParser::from_pest_pair(doc_type_v10_pair),
            ),
            Rule::membership_v10 => V10Document::Membership(
                membership::MembershipDocumentParser::from_pest_pair(doc_type_v10_pair),
            ),
            Rule::cert_v10 => V10Document::Certification(Box::new(
                certification::CertificationDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            Rule::revoc_v10 => V10Document::Revocation(Box::new(
                revocation::RevocationDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            Rule::tx_v10 => V10Document::Transaction(Box::new(
                transaction::TransactionDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            _ => panic!("unexpected rule: {:?}", doc_type_v10_pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

/// Trait for a compact V10 document.
pub trait CompactTextDocument: Sized + Clone {
    /// Generate document compact text.
    /// the compact format is the one used in the blocks.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn as_compact_text(&self) -> String;
}

impl<D: TextDocument> CompactTextDocument for TextDocumentFormat<D> {
    fn as_compact_text(&self) -> String {
        match *self {
            TextDocumentFormat::Complete(ref doc) => doc.generate_compact_text(),
            TextDocumentFormat::Compact(ref doc) => doc.as_compact_text(),
        }
    }
}

/// Trait for a V10 document.
pub trait TextDocument: Document<PublicKey = PubKey, CurrencyType = str> {
    /// Type of associated compact document.
    type CompactTextDocument_: CompactTextDocument;

    /// Return document as text.
    fn as_text(&self) -> &str;

    /// Return document as text without signature.
    fn as_text_without_signature(&self) -> &str {
        let text = self.as_text();
        let mut lines: Vec<&str> = self.as_text().split('\n').collect();
        let sigs = self.signatures();
        let mut sigs_str_len = sigs.len() - 1;
        for _ in sigs {
            sigs_str_len += lines.pop().unwrap_or("").len();
        }
        &text[0..(text.len() - sigs_str_len)]
    }

    /*/// Return document as text with leading signatures.
    fn as_text_with_signatures(&self) -> String {
        let mut text = self.as_text().to_string();
    
        for sig in self.signatures() {
            text = format!("{}{}\n", text, sig.to_base64());
        }
    
        text
    }*/

    /// Generate compact document.
    /// the compact format is the one used in the blocks.
    /// - Don't contains leading signatures
    fn to_compact_document(&self) -> Self::CompactTextDocument_;

    /// Generate document compact text.
    /// the compact format is the one used in the blocks.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn generate_compact_text(&self) -> String {
        self.to_compact_document().as_compact_text()
    }
}

/// Trait for a V10 document builder.
pub trait TextDocumentBuilder: DocumentBuilder {
    /// Generate document text.
    ///
    /// - Don't contains leading signatures
    /// - Contains line breaks on all line.
    fn generate_text(&self) -> String;

    /// Generate final document with signatures, and also return them in an array.
    ///
    /// Returns :
    ///
    /// - Text without signatures
    /// - Signatures
    fn build_signed_text(&self, private_keys: Vec<PrivKey>) -> (String, Vec<Sig>) {
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

/// V10 Documents in separated parts
#[derive(Debug, Clone)]
pub struct V10DocumentParts {
    /// Whole document in text
    pub doc: String,
    /// Payload
    pub body: String,
    /// Currency
    pub currency: String,
    /// Signatures
    pub signatures: Vec<Sig>,
}

/*/// A V10 document parser.
#[derive(Debug, Clone, Copy)]
pub struct V10DocumentParser;

impl<'a> DocumentParser<&'a str, V10Document, TextDocumentParseError> for V10DocumentParser {
    fn parse(source: &'a str) -> Result<V10Document, TextDocumentParseError> {
        /*match DocumentsParser::parse(Rule::document_v10, source) {
            Ok(mut source_ast) => {
                let doc_v10_ast = source_ast.next().unwrap(); // get and unwrap the `document_v10` rule; never fails
                let doc_type_v10_ast = doc_v10_ast.into_inner().next().unwrap(); // get and unwrap the `{DOC_TYPE}_v10` rule; never fails
        
                match doc_type_v10_ast.as_rule() {
                    Rule::idty_v10 => IdentityDocumentParser::parse_standard(doc_type_v10_ast.as_str(), "", currency, vec![]),
                    Rule::membership_v10 => MembershipDocumentParser::parse_standard(doc_type_v10_ast.as_str(), "", currency, vec![]),
                    Rule::cert_v10 => CertificationDocumentParser::parse_standard(doc_type_v10_ast.as_str(), "", currency, vec![]),
                    Rule::revoc_v10 => RevocationDocumentParser::parse_standard(doc_type_v10_ast.as_str(), "", currency, vec![]),
                    Rule::tx_v10 => TransactionDocumentParser::parse_standard(doc_type_v10_ast.as_str(), "", currency, vec![]),
                }
            }
            Err(_) => Err(TextDocumentParseError::InvalidWrapperFormat()),
        }*/
if let Some(caps) = DOCUMENT_REGEX.captures(source) {
let doctype = &caps["type"];
let currency = &caps["currency"];

// TODO : Improve error handling of Signature::from_base64 failure

match doctype {
"Identity" => IdentityDocumentParser::parse_standard(source, currency),
"Membership" => MembershipDocumentParser::parse_standard(source, currency),
"Certification" => CertificationDocumentParser::parse_standard(source, currency),
"Revocation" => RevocationDocumentParser::parse_standard(source, currency),
"Transaction" => TransactionDocumentParser::parse_standard(source, currency),
_ => Err(TextDocumentParseError::UnknownDocumentType(
doctype.to_string(),
)),
}
} else {
Err(TextDocumentParseError::InvalidWrapperFormat())
}
}
}*/

#[cfg(test)]
mod tests {
    use super::certification::CertificationDocumentParser;
    use super::identity::IdentityDocumentParser;
    use super::membership::MembershipDocumentParser;
    use super::revocation::RevocationDocumentParser;
    use super::transaction::TransactionDocumentParser;
    use super::*;
    use dup_crypto::keys::*;

    // simple text document for signature testing
    #[derive(Debug, Clone)]
    struct PlainTextDocument {
        pub text: &'static str,
        pub issuers: Vec<PubKey>,
        pub signatures: Vec<Sig>,
    }

    impl Document for PlainTextDocument {
        type PublicKey = PubKey;
        type CurrencyType = str;

        fn version(&self) -> u16 {
            unimplemented!()
        }

        fn currency(&self) -> &str {
            unimplemented!()
        }

        fn blockstamp(&self) -> Blockstamp {
            unimplemented!()
        }

        fn issuers(&self) -> &Vec<PubKey> {
            &self.issuers
        }

        fn signatures(&self) -> &Vec<Sig> {
            &self.signatures
        }

        fn as_bytes(&self) -> &[u8] {
            self.text.as_bytes()
        }
    }

    #[test]
    fn verify_signatures() {
        let text = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        // good pair
        let issuer1 = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let sig1 = Sig::Ed25519(
            ed25519::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
                 mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
            )
            .unwrap(),
        );

        // incorrect pair
        let issuer2 = PubKey::Ed25519(
            ed25519::PublicKey::from_base58("DNann1Lh55eZMEDXeYt32bzHbA3NJR46DeQYCS2qQdLV")
                .unwrap(),
        );

        let sig2 = Sig::Ed25519(
            ed25519::Signature::from_base64(
                "1eubHHbuNfilHHH0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
                 mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
            )
            .unwrap(),
        );

        {
            let doc = PlainTextDocument {
                text,
                issuers: vec![issuer1],
                signatures: vec![sig1],
            };

            assert_eq!(doc.verify_signatures(), VerificationResult::Valid());
        }

        {
            let doc = PlainTextDocument {
                text,
                issuers: vec![issuer1],
                signatures: vec![sig2],
            };

            assert_eq!(
                doc.verify_signatures(),
                VerificationResult::Invalid(vec![0])
            );
        }

        {
            let doc = PlainTextDocument {
                text,
                issuers: vec![issuer1, issuer2],
                signatures: vec![sig1],
            };

            assert_eq!(
                doc.verify_signatures(),
                VerificationResult::IncompletePairs(2, 1)
            );
        }

        {
            let doc = PlainTextDocument {
                text,
                issuers: vec![issuer1],
                signatures: vec![sig1, sig2],
            };

            assert_eq!(
                doc.verify_signatures(),
                VerificationResult::IncompletePairs(1, 2)
            );
        }
    }

    #[test]
    fn parse_identity_document() {
        let text = "Version: 10
Type: Identity
Currency: g1
Issuer: D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx
UniqueID: elois
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Ydnclvw76/JHcKSmU9kl9Ie0ne5/X8NYOqPqbGnufIK3eEPRYYdEYaQh+zffuFhbtIRjv6m/DkVLH5cLy/IyAg==";

        let doc = IdentityDocumentParser::parse(text).unwrap();
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }

    #[test]
    fn parse_membership_document() {
        let text = "Version: 10
Type: Membership
Currency: g1
Issuer: D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx
Block: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Membership: IN
UserID: elois
CertTS: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
FFeyrvYio9uYwY5aMcDGswZPNjGLrl8THn9l3EPKSNySD3SDSHjCljSfFEwb87sroyzJQoVzPwER0sW/cbZMDg==";

        let doc = MembershipDocumentParser::parse(text).unwrap();
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }

    #[test]
    fn parse_certification_document() {
        let text = "Version: 10
Type: Certification
Currency: g1
Issuer: 2sZF6j2PkxBDNAqUde7Dgo5x3crkerZpQ4rBqqJGn8QT
IdtyIssuer: 7jzkd8GiFnpys4X7mP78w2Y3y3kwdK6fVSLEaojd3aH9
IdtyUniqueID: fbarbut
IdtyTimestamp: 98221-000000575AC04F5164F7A307CDB766139EA47DD249E4A2444F292BC8AAB408B3
IdtySignature: DjeipIeb/RF0tpVCnVnuw6mH1iLJHIsDfPGLR90Twy3PeoaDz6Yzhc/UjLWqHCi5Y6wYajV0dNg4jQRUneVBCQ==
CertTimestamp: 99956-00000472758331FDA8388E30E50CA04736CBFD3B7C21F34E74707107794B56DD
Hkps1QU4HxIcNXKT8YmprYTVByBhPP1U2tIM7Z8wENzLKIWAvQClkAvBE7pW9dnVa18sJIJhVZUcRrPAZfmjBA==";

        let doc = CertificationDocumentParser::parse(text).unwrap();
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }

    #[test]
    fn parse_revocation_document() {
        let text = "Version: 10
Type: Revocation
Currency: g1
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
IdtyUniqueID: tic
IdtyTimestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
IdtySignature: 1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==
XXOgI++6qpY9O31ml/FcfbXCE6aixIrgkT5jL7kBle3YOMr+8wrp7Rt+z9hDVjrNfYX2gpeJsuMNfG4T/fzVDQ==";

        let doc = RevocationDocumentParser::parse(text).unwrap();
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }

    #[test]
    fn parse_transaction_document() {
        let text = "Version: 10
Type: Transaction
Currency: g1
Blockstamp: 107702-0000017CDBE974DC9A46B89EE7DC2BEE4017C43A005359E0853026C21FB6A084
Locktime: 0
Issuers:
Do6Y6nQ2KTo5fB4MXbSwabXVmXHxYRB9UUAaTPKn1XqC
Inputs:
1002:0:D:Do6Y6nQ2KTo5fB4MXbSwabXVmXHxYRB9UUAaTPKn1XqC:104937
1002:0:D:Do6Y6nQ2KTo5fB4MXbSwabXVmXHxYRB9UUAaTPKn1XqC:105214
Unlocks:
0:SIG(0)
1:SIG(0)
Outputs:
2004:0:SIG(DTgQ97AuJ8UgVXcxmNtULAs8Fg1kKC1Wr9SAS96Br9NG)
Comment: c est pour 2 mois d adhesion ressourcerie
lnpuFsIymgz7qhKF/GsZ3n3W8ZauAAfWmT4W0iJQBLKJK2GFkesLWeMj/+GBfjD6kdkjreg9M6VfkwIZH+hCCQ==";

        let doc = TransactionDocumentParser::parse(text).unwrap();
        println!("Doc : {:?}", doc);
        assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
    }
}

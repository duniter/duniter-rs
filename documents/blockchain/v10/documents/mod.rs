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

extern crate crypto;

use self::crypto::digest::Digest;

use blockchain::v10::documents::identity::IdentityDocumentParser;
use blockchain::{Document, DocumentBuilder, DocumentParser};
use duniter_crypto::keys::{ed25519, Signature};
use regex::Regex;

pub mod block;
pub mod certification;
pub mod identity;
pub mod membership;
pub mod revocation;
pub mod transaction;

pub use blockchain::v10::documents::block::BlockDocument;
pub use blockchain::v10::documents::certification::{
    CertificationDocument, CertificationDocumentParser,
};
pub use blockchain::v10::documents::identity::{IdentityDocument, IdentityDocumentBuilder};
pub use blockchain::v10::documents::membership::{MembershipDocument, MembershipDocumentParser};
pub use blockchain::v10::documents::revocation::{RevocationDocument, RevocationDocumentParser};
pub use blockchain::v10::documents::transaction::{
    TransactionDocument, TransactionDocumentBuilder, TransactionDocumentParser,
};

use std::marker::Sized;

// Use of lazy_static so the regex is only compiled at first use.
lazy_static! {
    static ref DOCUMENT_REGEX: Regex = Regex::new(
        "^(?P<doc>Version: (?P<version>[0-9]+)\n\
         Type: (?P<type>[[:alpha:]]+)\n\
         Currency: (?P<currency>[[:alnum:] _-]+)\n\
         (?P<body>(?:.*\n)+?))\
         (?P<sigs>([[:alnum:]+/=]+\n)*([[:alnum:]+/=]+))$"
    ).unwrap();
    static ref SIGNATURES_REGEX: Regex = Regex::new("[[:alnum:]+/=]+\n?").unwrap();
}

#[derive(Debug, Clone)]
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
///
/// > TODO Add wrapped types in enum variants.
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
pub trait TextDocument: Document<PublicKey = ed25519::PublicKey, CurrencyType = str> {
    /// Type of associated compact document.
    type CompactTextDocument_: CompactTextDocument;

    /// Return document as text.
    fn as_text(&self) -> &str;

    /// Return sha256 hash of text document
    fn hash<H: Digest>(&self, digest: &mut H) -> String {
        digest.input_str(self.as_text());
        digest.result_str()
    }

    /// Return document as text with leading signatures.
    fn as_text_with_signatures(&self) -> String {
        let mut text = self.as_text().to_string();

        for sig in self.signatures() {
            text = format!("{}{}\n", text, sig.to_base64());
        }

        text
    }

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
    fn build_signed_text(
        &self,
        private_keys: Vec<ed25519::PrivateKey>,
    ) -> (String, Vec<ed25519::Signature>) {
        use duniter_crypto::keys::PrivateKey;

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

/// List of possible errors while parsing.
#[derive(Debug, Clone)]
pub enum V10DocumentParsingError {
    /// The given source don't have a valid document format.
    InvalidWrapperFormat(),
    /// The given source don't have a valid specific document format (document type).
    InvalidInnerFormat(String),
    /// Type fields contains an unknown document type.
    UnknownDocumentType(String),
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
    pub signatures: Vec<ed25519::Signature>,
}

trait StandardTextDocumentParser {
    fn parse_standard(
        doc: &str,
        body: &str,
        currency: &str,
        signatures: Vec<ed25519::Signature>,
    ) -> Result<V10Document, V10DocumentParsingError>;
}

/// A V10 document parser.
#[derive(Debug, Clone, Copy)]
pub struct V10DocumentParser;

impl<'a> DocumentParser<&'a str, V10Document, V10DocumentParsingError> for V10DocumentParser {
    fn parse(source: &'a str) -> Result<V10Document, V10DocumentParsingError> {
        if let Some(caps) = DOCUMENT_REGEX.captures(source) {
            let doctype = &caps["type"];
            let doc = &caps["doc"];
            let currency = &caps["currency"];
            let body = &caps["body"];
            let sigs = SIGNATURES_REGEX
                .captures_iter(&caps["sigs"])
                .map(|capture| ed25519::Signature::from_base64(&capture[0]).unwrap())
                .collect::<Vec<_>>();

            // TODO : Improve error handling of Signature::from_base64 failure

            match doctype {
                "Identity" => IdentityDocumentParser::parse_standard(doc, body, currency, sigs),
                "Membership" => MembershipDocumentParser::parse_standard(doc, body, currency, sigs),
                "Certification" => {
                    CertificationDocumentParser::parse_standard(doc, body, currency, sigs)
                }
                "Revocation" => RevocationDocumentParser::parse_standard(doc, body, currency, sigs),
                "Transaction" => {
                    TransactionDocumentParser::parse_standard(doc, body, currency, sigs)
                }
                _ => Err(V10DocumentParsingError::UnknownDocumentType(
                    doctype.to_string(),
                )),
            }
        } else {
            Err(V10DocumentParsingError::InvalidWrapperFormat())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain::{Document, VerificationResult};

    #[test]
    fn document_regex() {
        assert!(DOCUMENT_REGEX.is_match(
            "Version: 10
Type: Transaction
Currency: beta_brousouf
Blockstamp: 204-00003E2B8A35370BA5A7064598F628A62D4E9EC1936BE8651CE9A85F2E06981B
Locktime: 0
Issuers:
HsLShAtzXTVxeUtQd7yi5Z5Zh4zNvbu8sTEZ53nfKcqY
CYYjHsNyg3HMRMpTHqCJAN9McjH5BwFLmDKGV3PmCuKp
9WYHTavL1pmhunFCzUwiiq4pXwvgGG5ysjZnjz9H8yB
Inputs:
40:2:T:6991C993631BED4733972ED7538E41CCC33660F554E3C51963E2A0AC4D6453D3:2
70:2:T:3A09A20E9014110FD224889F13357BAB4EC78A72F95CA03394D8CCA2936A7435:8
20:2:D:HsLShAtzXTVxeUtQd7yi5Z5Zh4zNvbu8sTEZ53nfKcqY:46
70:2:T:A0D9B4CDC113ECE1145C5525873821398890AE842F4B318BD076095A23E70956:3
20:2:T:67F2045B5318777CC52CD38B424F3E40DDA823FA0364625F124BABE0030E7B5B:5
15:2:D:9WYHTavL1pmhunFCzUwiiq4pXwvgGG5ysjZnjz9H8yB:46
Unlocks:
0:SIG(0)
1:XHX(7665798292)
2:SIG(0)
3:SIG(0) SIG(2)
4:SIG(0) SIG(1) SIG(2)
5:SIG(2)
Outputs:
120:2:SIG(BYfWYFrsyjpvpFysgu19rGK3VHBkz4MqmQbNyEuVU64g)
146:2:SIG(DSz4rgncXCytsUMW2JU2yhLquZECD2XpEkpP9gG5HyAx)
49:2:(SIG(6DyGr5LFtFmbaJYRvcs9WmBsr4cbJbJ1EV9zBbqG7A6i)\
 || XHX(3EB4702F2AC2FD3FA4FDC46A4FC05AE8CDEE1A85))
Comment: -----@@@----- (why not this comment?)
42yQm4hGTJYWkPg39hQAUgP6S6EQ4vTfXdJuxKEHL1ih6YHiDL2hcwrFgBHjXLRgxRhj2VNVqqc6b4JayKqTE14r
2D96KZwNUvVtcapQPq2mm7J9isFcDCfykwJpVEZwBc7tCgL4qPyu17BT5ePozAE9HS6Yvj51f62Mp4n9d9dkzJoX
2XiBDpuUdu6zCPWGzHXXy8c4ATSscfFQG9DjmqMZUxDZVt1Dp4m2N5oHYVUfoPdrU9SLk4qxi65RNrfCVnvQtQJk"
        ));

        assert!(DOCUMENT_REGEX.is_match(
            "Version: 10
Type: Certification
Currency: beta_brousouf
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
IdtyIssuer: HgTTJLAQ5sqfknMq7yLPZbehtuLSsKj9CxWN7k8QvYJd
IdtyUniqueID: lolcat
IdtyTimestamp: 32-DB30D958EE5CB75186972286ED3F4686B8A1C2CD
IdtySignature: J3G9oM5AKYZNLAB5Wx499w61NuUoS57JVccTShUb\
GpCMjCqj9yXXqNq7dyZpDWA6BxipsiaMZhujMeBfCznzyci
CertTimestamp: 36-1076F10A7397715D2BEE82579861999EA1F274AC
SoKwoa8PFfCDJWZ6dNCv7XstezHcc2BbKiJgVDXv82R5zYR83nis9dShLgWJ5w48noVUHimdngzYQneNYSMV3rk"
        ));
    }

    #[test]
    fn signatures_regex() {
        assert_eq!(
            SIGNATURES_REGEX
                .captures_iter(
                    "
42yQm4hGTJYWkPg39hQAUgP6S6EQ4vTfXdJuxKEHL1ih6YHiDL2hcwrFgBHjXLRgxRhj2VNVqqc6b4JayKqTE14r
2D96KZwNUvVtcapQPq2mm7J9isFcDCfykwJpVEZwBc7tCgL4qPyu17BT5ePozAE9HS6Yvj51f62Mp4n9d9dkzJoX
2XiBDpuUdu6zCPWGzHXXy8c4ATSscfFQG9DjmqMZUxDZVt1Dp4m2N5oHYVUfoPdrU9SLk4qxi65RNrfCVnvQtQJk"
                )
                .count(),
            3
        );

        assert_eq!(
            SIGNATURES_REGEX
                .captures_iter(
                    "
42yQm4hGTJYWkPg39hQAUgP6S6EQ4vTfXdJuxKEHL1ih6YHiDL2hcwrFgBHjXLRgxRhj2VNVqqc6b4JayKqTE14r
2XiBDpuUdu6zCPWGzHXXy8c4ATSscfFQG9DjmqMZUxDZVt1Dp4m2N5oHYVUfoPdrU9SLk4qxi65RNrfCVnvQtQJk"
                )
                .count(),
            2
        );
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

        let doc = V10DocumentParser::parse(text).unwrap();
        if let V10Document::Identity(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
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

        let doc = V10DocumentParser::parse(text).unwrap();
        if let V10Document::Membership(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
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

        let doc = V10DocumentParser::parse(text).unwrap();
        if let V10Document::Certification(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
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

        let doc = V10DocumentParser::parse(text).unwrap();
        if let V10Document::Revocation(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
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

        let doc = V10DocumentParser::parse(text).unwrap();
        if let V10Document::Transaction(doc) = doc {
            println!("Doc : {:?}", doc);
            assert_eq!(doc.verify_signatures(), VerificationResult::Valid())
        } else {
            panic!("Wrong document type");
        }
    }
}

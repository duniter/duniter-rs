//  Copyright (C) 2018  The Durs Project Developers.
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

//! Implements the Durs blockchain Documents.

use crate::documents::block::*;
use crate::documents::certification::*;
use crate::documents::identity::*;
use crate::documents::membership::*;
use crate::documents::revocation::*;
use crate::documents::transaction::*;
use crate::Rule;
use crate::*;

use pest::iterators::Pair;
use pest::Parser;

pub mod block;
pub mod certification;
pub mod identity;
pub mod membership;
pub mod revocation;
pub mod transaction;

/// Document of DUBP (DUniter Blockhain Protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DUBPDocument {
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

/// List of stringified document types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DUBPDocumentStr {
    /// Block document (not yet implemented)
    Block(),

    /// Transaction document.
    Transaction(Box<TransactionDocumentStringified>),

    /// Identity document.
    Identity(IdentityStringDocument),

    /// Membership document.
    Membership(MembershipStringDocument),

    /// Certification document.
    Certification(Box<CertificationStringDocument>),

    /// Revocation document.
    Revocation(Box<RevocationStringDocument>),
}

impl ToStringObject for DUBPDocument {
    type StringObject = DUBPDocumentStr;
    /// Transforms an object into a json object
    fn to_string_object(&self) -> Self::StringObject {
        match *self {
            DUBPDocument::Block(_) => unimplemented!(),
            DUBPDocument::Identity(ref doc) => DUBPDocumentStr::Identity(doc.to_string_object()),
            DUBPDocument::Membership(ref doc) => {
                DUBPDocumentStr::Membership(doc.to_string_object())
            }
            DUBPDocument::Certification(ref doc) => {
                DUBPDocumentStr::Certification(Box::new(doc.to_string_object()))
            }
            DUBPDocument::Revocation(ref doc) => {
                DUBPDocumentStr::Revocation(Box::new(doc.to_string_object()))
            }
            DUBPDocument::Transaction(ref doc) => {
                DUBPDocumentStr::Transaction(Box::new(doc.to_string_object()))
            }
        }
    }
}

impl TextDocumentParser<Rule> for DUBPDocument {
    type DocumentType = DUBPDocument;

    fn parse(doc: &str) -> Result<DUBPDocument, TextDocumentParseError> {
        match DocumentsParser::parse(Rule::document, doc) {
            Ok(mut doc_pairs) => Ok(DUBPDocument::from_pest_pair(doc_pairs.next().unwrap())), // get and unwrap the `document` rule; never fails
            Err(pest_error) => Err(TextDocumentParseError::PestError(format!("{}", pest_error))),
        }
    }
    fn from_pest_pair(pair: Pair<Rule>) -> Self::DocumentType {
        let doc_vx_pair = pair.into_inner().next().unwrap(); // get and unwrap the `document_vX` rule; never fails

        match doc_vx_pair.as_rule() {
            Rule::document_v10 => DUBPDocument::from_pest_pair_v10(doc_vx_pair),
            _ => panic!("unexpected rule: {:?}", doc_vx_pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

impl DUBPDocument {
    pub fn from_pest_pair_v10(pair: Pair<Rule>) -> DUBPDocument {
        let doc_type_v10_pair = pair.into_inner().next().unwrap(); // get and unwrap the `{DOC_TYPE}_v10` rule; never fails

        match doc_type_v10_pair.as_rule() {
            Rule::idty_v10 => DUBPDocument::Identity(
                identity::IdentityDocumentParser::from_pest_pair(doc_type_v10_pair),
            ),
            Rule::membership_v10 => DUBPDocument::Membership(
                membership::MembershipDocumentParser::from_pest_pair(doc_type_v10_pair),
            ),
            Rule::cert_v10 => DUBPDocument::Certification(Box::new(
                certification::CertificationDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            Rule::revoc_v10 => DUBPDocument::Revocation(Box::new(
                revocation::RevocationDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            Rule::tx_v10 => DUBPDocument::Transaction(Box::new(
                transaction::TransactionDocumentParser::from_pest_pair(doc_type_v10_pair),
            )),
            _ => panic!("unexpected rule: {:?}", doc_type_v10_pair.as_rule()), // Grammar ensures that we never reach this line
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::blockstamp::Blockstamp;
    use crate::*;

    use super::certification::CertificationDocumentParser;
    use super::identity::IdentityDocumentParser;
    use super::membership::MembershipDocumentParser;
    use super::revocation::RevocationDocumentParser;
    use super::transaction::TransactionDocumentParser;
    use super::*;

    use dup_crypto::keys::*;

    // simple text document for signature testing
    #[derive(Debug, Clone, PartialEq, Eq)]
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

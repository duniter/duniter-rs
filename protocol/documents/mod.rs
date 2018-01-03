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

//! Provide wrappers around Duniter documents.

pub mod block10;

use std::fmt::Debug;
use duniter_keys::{PublicKey, Signature};

/// Common top-level document trait.
///
/// Provide commun methods for any documents of any protocol version.
///
/// Document trait is generic about its signature scheme.
///
/// > Design choice : allow only ed25519 for protocol 10 and many differents
/// schemes for protocol 11 through a proxy type.
pub trait Document<PK, S>: Debug
where
    PK: PublicKey<Signature = S>,
    S: Signature,
{
    /// Get document version.
    fn version(&self) -> u16;

    /// Get document currency.
    fn currency(&self) -> &str;

    /// Iterate over document issuers.
    fn issuers(&self) -> &Vec<PK>;

    /// Iterate over document signatures.
    fn signatures(&self) -> &Vec<S>;

    /// Get document as bytes for signature verification.
    fn as_bytes(&self) -> &[u8];

    /// Verify signatures of document content (as text format)
    fn verify_signatures(&self) -> VerificationResult {
        let issuers_count = self.issuers().len();
        let signatures_count = self.signatures().len();

        if issuers_count != signatures_count {
            VerificationResult::IncompletePairs(issuers_count, signatures_count)
        } else {
            let issuers = self.issuers();
            let signatures = self.signatures();
            let mismatches: Vec<_> = issuers
                .iter()
                .zip(signatures.iter())
                .enumerate()
                .filter(|&(_, (key, signature))| {
                    !key.verify(self.as_bytes(), signature)
                })
                .map(|(i, _)| i)
                .collect();

            if mismatches.is_empty() {
                VerificationResult::Valid()
            } else {
                VerificationResult::Invalid(mismatches)
            }
        }
    }
}

/// List of possible results for signature verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// All signatures are valid.
    Valid(),
    /// Not same amount of issuers and signatures.
    /// (issuers count, signatures count)
    IncompletePairs(usize, usize),
    /// Signatures don't match.
    /// List of mismatching pairs indexes.
    Invalid(Vec<usize>),
}

/// Trait allowing access to the document through it's proper protocol version.
///
/// This trait is generic over `VersionEnum` providing all supported protocol version variants.
///
/// A lifetime is specified to allow enum variants to hold references to the document.
pub trait ToProtocolDocument<'a, PK, S, VersionEnum>: Document<PK, S>
where
    PK: PublicKey<Signature = S>,
    S: Signature,
{
    /// Get a protocol-specific document wrapped in an enum variant.
    fn associated_protocol(&'a self) -> VersionEnum;
}

/// Trait converting a document to a specialized document wrapped in an enum variant.
///
/// This trait is generic over an `TypeEnum` specific to the protcol version.
///
/// A lifetime is specified to allow enum variants to hold references to the document.
pub trait ToSpecializedDocument<'a, PK, S, TypeEnum: 'a>: Document<PK, S>
where
    PK: PublicKey<Signature=S>,
    S: Signature, {
/// Get specialized document wrapped in an enum variant.
    fn specialize(&'a self) -> TypeEnum;
}

/// List of blockchain protocol versions.
#[derive(Debug)]
pub enum BlockchainProtocolVersion<'a> {
    /// Version 10.
    V10(&'a block10::TextDocument<'a>),
    /// Version 11. (not done yet, but defined for tests)
    V11(),
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_keys::ed25519;

    // simple text document for signature testing
    #[derive(Debug, Clone)]
    struct PlainTextDocument {
        pub text: &'static str,
        pub issuers: Vec<ed25519::PublicKey>,
        pub signatures: Vec<ed25519::Signature>,
    }

    impl Document<ed25519::PublicKey, ed25519::Signature> for PlainTextDocument {
        fn version(&self) -> u16 {
            unimplemented!()
        }

        fn currency(&self) -> &str {
            unimplemented!()
        }

        fn issuers(&self) -> &Vec<ed25519::PublicKey> {
            &self.issuers
        }

        fn signatures(&self) -> &Vec<ed25519::Signature> {
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
        let issuer1 = ed25519::PublicKey::from_base58(
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV",
        ).unwrap();

        let sig1 = ed25519::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
            mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap();

        // incorrect pair
        let issuer2 = ed25519::PublicKey::from_base58(
            "DNann1Lh55eZMEDXeYt32bzHbA3NJR46DeQYCS2qQdLV",
        ).unwrap();

        let sig2 = ed25519::Signature::from_base64(
            "1eubHHbuNfilHHH0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
            mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap();

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
}

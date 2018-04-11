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

//! Provide wrappers around Duniter documents and events.

use std::fmt::Debug;

use duniter_crypto::keys::{PrivateKey, PublicKey};

use super::Blockstamp;

pub mod v10;

/// List of blockchain protocol versions.
#[derive(Debug, Clone)]
pub enum BlockchainProtocol {
    /// Version 10.
    V10(Box<v10::documents::V10Document>),
    /// Version 11. (not done yet, but defined for tests)
    V11(),
}

/// trait providing commun methods for any documents of any protocol version.
///
/// # Design choice
///
/// Allow only ed25519 for protocol 10 and many differents
/// schemes for protocol 11 through a proxy type.
pub trait Document: Debug + Clone {
    /// Type of the `PublicKey` used by the document.
    type PublicKey: PublicKey;
    /// Data type of the currency code used by the document.
    type CurrencyType: ?Sized;

    /// Get document version.
    fn version(&self) -> u16;

    /// Get document currency.
    fn currency(&self) -> &Self::CurrencyType;

    /// Get document blockstamp
    fn blockstamp(&self) -> Blockstamp;

    /// Iterate over document issuers.
    fn issuers(&self) -> &Vec<Self::PublicKey>;

    /// Iterate over document signatures.
    fn signatures(&self) -> &Vec<<Self::PublicKey as PublicKey>::Signature>;

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
                .zip(signatures)
                .enumerate()
                .filter(|&(_, (key, signature))| !key.verify(self.as_bytes(), signature))
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
/// This trait is generic over `P` providing all supported protocol version variants.
///
/// A lifetime is specified to allow enum variants to hold references to the document.
pub trait IntoSpecializedDocument<P> {
    /// Get a protocol-specific document wrapped in an enum variant.
    fn into_specialized(self) -> P;
}

/// Trait helper for building new documents.
pub trait DocumentBuilder {
    /// Type of the builded document.
    type Document: Document;

    /// Type of the private keys signing the documents.
    type PrivateKey: PrivateKey<
        Signature = <<Self::Document as Document>::PublicKey as PublicKey>::Signature,
    >;

    /// Build a document with provided signatures.
    fn build_with_signature(
        &self,
        signatures: Vec<<<Self::Document as Document>::PublicKey as PublicKey>::Signature>,
    ) -> Self::Document;

    /// Build a document and sign it with the private key.
    fn build_and_sign(&self, private_keys: Vec<Self::PrivateKey>) -> Self::Document;
}

/// Trait for a document parser from a `S` source
/// format to a `D` document. Will return the
/// parsed document or an `E` error.
pub trait DocumentParser<S, D, E> {
    /// Parse a source and return a document or an error.
    fn parse(source: S) -> Result<D, E>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use duniter_crypto::keys::{ed25519, Signature};

    // simple text document for signature testing
    #[derive(Debug, Clone)]
    struct PlainTextDocument {
        pub text: &'static str,
        pub issuers: Vec<ed25519::PublicKey>,
        pub signatures: Vec<ed25519::Signature>,
    }

    impl Document for PlainTextDocument {
        type PublicKey = ed25519::PublicKey;
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

//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! Implements the Dunitrust Documents Protocol.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate pest_derive;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate serde_derive;

pub mod blockstamp;
pub mod documents;
pub mod parsers;
pub mod text_document_traits;

use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use pest::iterators::Pair;
use pest::RuleType;
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Error, Formatter};
use std::net::AddrParseError;

pub use crate::blockstamp::{Blockstamp, PreviousBlockstamp};

#[derive(Parser)]
#[grammar = "documents_grammar.pest"]
/// Parser for Documents
struct DocumentsParser;

pub trait TextDocumentParser<R: RuleType> {
    /// Type of document generated by the parser
    type DocumentType;

    /// Parse text document from raw format
    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError>;
    /// Parse text document from pest pairs
    fn from_pest_pair(pairs: Pair<R>) -> Result<Self::DocumentType, TextDocumentParseError>;
    /// Parse text document from versioned pest pairs
    fn from_versioned_pest_pair(
        version: u16,
        pairs: Pair<R>,
    ) -> Result<Self::DocumentType, TextDocumentParseError>;
}

/// Error with pest parser (grammar)
#[derive(Debug, Clone, Eq, Fail, PartialEq)]
#[fail(display = "Grammar error: {}", _0)]
pub struct PestError(pub String);

impl<T: pest::RuleType> From<pest::error::Error<T>> for PestError {
    fn from(e: pest::error::Error<T>) -> Self {
        PestError(format!("{}", e))
    }
}

/// List of possible errors while parsing a text document.
#[derive(Debug, Clone, Eq, Fail, PartialEq)]
pub enum TextDocumentParseError {
    /// The given source don't have a valid specific document format (document type).
    #[fail(display = "TextDocumentParseError: Invalid inner format: {}", _0)]
    InvalidInnerFormat(String),
    /// Ip address parse error
    #[fail(display = "TextDocumentParseError: invalid ip: {}", _0)]
    IpAddrError(AddrParseError),
    /// Error with pest parser
    #[fail(display = "TextDocumentParseError: {}", _0)]
    PestError(PestError),
    /// Unexpected rule
    #[fail(display = "TextDocumentParseError: Unexpected rule: '{}'", _0)]
    UnexpectedRule(String),
    /// Unexpected version
    #[fail(display = "TextDocumentParseError: Unexpected version: '{}'", _0)]
    UnexpectedVersion(String),
    /// Unknown type
    #[fail(display = "TextDocumentParseError: UnknownType.")]
    UnknownType,
}

impl From<AddrParseError> for TextDocumentParseError {
    fn from(e: AddrParseError) -> Self {
        TextDocumentParseError::IpAddrError(e)
    }
}

impl From<PestError> for TextDocumentParseError {
    fn from(e: PestError) -> Self {
        TextDocumentParseError::PestError(e)
    }
}

impl<T: pest::RuleType> From<pest::error::Error<T>> for TextDocumentParseError {
    fn from(e: pest::error::Error<T>) -> Self {
        TextDocumentParseError::PestError(e.into())
    }
}

/// A block Id.
#[derive(Copy, Clone, Debug, Deserialize, Ord, PartialEq, PartialOrd, Eq, Hash, Serialize)]
pub struct BlockNumber(pub u32);

impl Display for BlockNumber {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

/// Wrapper of a block hash.
#[derive(Copy, Clone, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize)]
pub struct BlockHash(pub Hash);

impl Display for BlockHash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.0.to_hex())
    }
}

impl Debug for BlockHash {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "BlockHash({})", self)
    }
}

/// trait providing commun methods for any documents of any protocol version.
///
/// # Design choice
///
/// Allow only ed25519 for protocol 10 and many differents
/// schemes for protocol 11 through a proxy type.
pub trait Document: Debug + Clone + PartialEq + Eq {
    /// Type of the `PublicKey` used by the document.
    type PublicKey: PublicKey;

    /// Get document as bytes for signature verification.
    fn as_bytes(&self) -> &[u8];

    /// Get document blockstamp
    fn blockstamp(&self) -> Blockstamp;

    /// Get document currency name.
    fn currency(&self) -> &str;

    /// Iterate over document issuers.
    fn issuers(&self) -> &Vec<Self::PublicKey>;

    /// Some documents do not directly store the sequence of bytes that will be signed but generate
    // it on request, so these types of documents cannot provide a reference to the signed bytes.
    fn no_as_bytes(&self) -> bool {
        false
    }

    /// Get document to bytes for signature verification.
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Iterate over document signatures.
    fn signatures(&self) -> &Vec<<Self::PublicKey as PublicKey>::Signature>;

    /// Verify one signature
    #[inline]
    fn verify_one_signature(
        &self,
        public_key: &Self::PublicKey,
        signature: &<Self::PublicKey as PublicKey>::Signature,
    ) -> bool {
        if self.no_as_bytes() {
            public_key.verify(&self.to_bytes(), signature)
        } else {
            public_key.verify(self.as_bytes(), signature)
        }
    }

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
                .filter(|&(_, (key, signature))| !self.verify_one_signature(key, signature))
                .map(|(i, _)| i)
                .collect();

            if mismatches.is_empty() {
                VerificationResult::Valid()
            } else {
                VerificationResult::Invalid(mismatches)
            }
        }
    }

    /// Get document version.
    fn version(&self) -> u16;
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

/// Stringify a document
pub trait ToStringObject {
    type StringObject: Serialize;
    /// Transforms object fields into string
    fn to_string_object(&self) -> Self::StringObject;
}

/// Jsonify a document
pub trait ToJsonObject: ToStringObject {
    /// Convert to JSON String
    fn to_json_string(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string(&self.to_string_object())?)
    }
    /// Convert to JSON String pretty
    fn to_json_string_pretty(&self) -> Result<String, serde_json::Error> {
        Ok(serde_json::to_string_pretty(&self.to_string_object())?)
    }
}

impl<T: ToStringObject> ToJsonObject for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::UserDocumentDUBP;

    #[test]
    fn parse_dubp_document() {
        let text = "Version: 10
Type: Identity
Currency: g1
Issuer: D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx
UniqueID: elois
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
Ydnclvw76/JHcKSmU9kl9Ie0ne5/X8NYOqPqbGnufIK3eEPRYYdEYaQh+zffuFhbtIRjv6m/DkVLH5cLy/IyAg==";

        let doc = UserDocumentDUBP::parse(text).expect("Fail to parse UserDocumentDUBP !");
        println!("Doc : {:?}", doc);
    }
}

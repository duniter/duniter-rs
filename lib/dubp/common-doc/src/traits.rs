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

//! Define DUBP Documents Traits.

pub mod text;

use crate::blockstamp::Blockstamp;
use crate::errors::DocumentSigsErr;
use dup_crypto::keys::*;
use durs_common_tools::UsizeSer32;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;

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
    ) -> Result<(), SigError> {
        if self.no_as_bytes() {
            public_key.verify(&self.to_bytes(), signature)
        } else {
            public_key.verify(self.as_bytes(), signature)
        }
    }

    /// Verify signatures of document content
    fn verify_signatures(&self) -> Result<(), DocumentSigsErr> {
        let issuers_count = self.issuers().len();
        let signatures_count = self.signatures().len();

        if issuers_count != signatures_count {
            Err(DocumentSigsErr::IncompletePairs(
                issuers_count,
                signatures_count,
            ))
        } else {
            let issuers = self.issuers();
            let signatures = self.signatures();
            let mismatches: HashMap<usize, SigError> = issuers
                .iter()
                .zip(signatures)
                .enumerate()
                .filter_map(|(i, (key, signature))| {
                    if let Err(e) = self.verify_one_signature(key, signature) {
                        Some((i, e))
                    } else {
                        None
                    }
                })
                .collect();

            if mismatches.is_empty() {
                Ok(())
            } else {
                Err(DocumentSigsErr::Invalid(mismatches))
            }
        }
    }

    /// Get document version.
    fn version(&self) -> UsizeSer32;
}

/// Trait helper for building new documents.
pub trait DocumentBuilder {
    /// Type of the builded document.
    type Document: Document;

    /// Type of the signator signing the documents.
    type Signator: Signator<
        Signature = <<Self::Document as Document>::PublicKey as PublicKey>::Signature,
    >;

    /// Build a document with provided signatures.
    fn build_with_signature(
        &self,
        signatures: Vec<<<Self::Document as Document>::PublicKey as PublicKey>::Signature>,
    ) -> Self::Document;

    /// Build a document and sign it with the private key.
    fn build_and_sign(&self, signators: Vec<Self::Signator>) -> Self::Document;
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
    /// Generated string object
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

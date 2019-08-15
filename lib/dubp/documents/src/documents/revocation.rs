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

//! Wrappers around Revocation documents.

pub mod v10;

pub use v10::{
    CompactRevocationDocumentV10, CompactRevocationDocumentV10Stringified, RevocationDocumentV10,
    RevocationDocumentV10Stringified,
};

use crate::blockstamp::Blockstamp;
use crate::documents::*;

use dup_crypto::keys::*;
use pest::Parser;

/// Wrap an Revocation document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum RevocationDocument {
    /// Revocation document v10
    V10(RevocationDocumentV10),
}

/// Wrap an Compact Revocation document.
///
/// Must be created by a revocation document.
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CompactRevocationDocument {
    /// Compact revocation document v10
    V10(CompactRevocationDocumentV10),
}

impl Document for RevocationDocument {
    type PublicKey = PubKey;

    #[inline]
    fn version(&self) -> u16 {
        match self {
            RevocationDocument::V10(_) => 10u16,
        }
    }

    #[inline]
    fn currency(&self) -> &str {
        match self {
            RevocationDocument::V10(revoc_v10) => revoc_v10.currency(),
        }
    }

    #[inline]
    fn blockstamp(&self) -> Blockstamp {
        match self {
            RevocationDocument::V10(revoc_v10) => revoc_v10.blockstamp(),
        }
    }

    #[inline]
    fn issuers(&self) -> &Vec<PubKey> {
        match self {
            RevocationDocument::V10(revoc_v10) => revoc_v10.issuers(),
        }
    }

    #[inline]
    fn signatures(&self) -> &Vec<Sig> {
        match self {
            RevocationDocument::V10(revoc_v10) => revoc_v10.signatures(),
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            RevocationDocument::V10(revoc_v10) => revoc_v10.as_bytes(),
        }
    }
}

/// Revocation document parser
#[derive(Debug, Clone, Copy)]
pub struct RevocationDocumentParser;

impl TextDocumentParser<Rule> for RevocationDocumentParser {
    type DocumentType = RevocationDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut revoc_pairs = DocumentsParser::parse(Rule::revoc, doc)?;
        let revoc_pair = revoc_pairs.next().unwrap(); // get and unwrap the `revoc` rule; never fails
        Self::from_pest_pair(revoc_pair)
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let revoc_vx_pair = pair.into_inner().next().unwrap(); // get and unwrap the `revoc_vX` rule; never fails

        match revoc_vx_pair.as_rule() {
            Rule::revoc_v10 => Self::from_versioned_pest_pair(10, revoc_vx_pair),
            _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "{:#?}",
                revoc_vx_pair.as_rule()
            ))),
        }
    }
    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            10 => Ok(RevocationDocument::V10(
                RevocationDocumentV10::from_pest_pair(pair)?,
            )),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RevocationDocumentStringified {
    V10(RevocationDocumentV10Stringified),
}

impl ToStringObject for RevocationDocument {
    type StringObject = RevocationDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            RevocationDocument::V10(idty) => {
                RevocationDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CompactRevocationDocumentStringified {
    V10(CompactRevocationDocumentV10Stringified),
}

impl ToStringObject for CompactRevocationDocument {
    type StringObject = CompactRevocationDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            CompactRevocationDocument::V10(doc) => {
                CompactRevocationDocumentStringified::V10(doc.to_string_object())
            }
        }
    }
}

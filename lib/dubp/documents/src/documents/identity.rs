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

//! Wrappers around Identity documents.

pub mod v10;

use crate::documents::*;
use dubp_common_doc::blockstamp::Blockstamp;
use dubp_common_doc::parser::{DocumentsParser, TextDocumentParseError, TextDocumentParser};
use dubp_common_doc::traits::{Document, ToStringObject};
use dup_crypto::keys::*;

pub use v10::{IdentityDocumentV10, IdentityDocumentV10Stringified};

/// Identity document
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub enum IdentityDocument {
    /// Identity document V10
    V10(IdentityDocumentV10),
}

impl Document for IdentityDocument {
    type PublicKey = PubKey;

    #[inline]
    fn version(&self) -> u16 {
        match self {
            IdentityDocument::V10(_) => 10u16,
        }
    }

    #[inline]
    fn currency(&self) -> &str {
        match self {
            IdentityDocument::V10(idty_v10) => idty_v10.currency(),
        }
    }

    #[inline]
    fn blockstamp(&self) -> Blockstamp {
        match self {
            IdentityDocument::V10(idty_v10) => idty_v10.blockstamp(),
        }
    }

    #[inline]
    fn issuers(&self) -> &Vec<PubKey> {
        match self {
            IdentityDocument::V10(idty_v10) => idty_v10.issuers(),
        }
    }

    #[inline]
    fn signatures(&self) -> &Vec<Sig> {
        match self {
            IdentityDocument::V10(idty_v10) => idty_v10.signatures(),
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            IdentityDocument::V10(idty_v10) => idty_v10.as_bytes(),
        }
    }
}

/// Identity document parser
#[derive(Debug, Clone, Copy)]
pub struct IdentityDocumentParser;

impl TextDocumentParser<Rule> for IdentityDocumentParser {
    type DocumentType = IdentityDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut doc_pairs = DocumentsParser::parse(Rule::idty, doc)?;
        let idty_pair = doc_pairs.next().unwrap(); // get and unwrap the `idty` rule; never fails
        Self::from_pest_pair(idty_pair)
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let idty_vx_pair = pair.into_inner().next().unwrap(); // get and unwrap the `idty_vx` rule; never fails

        match idty_vx_pair.as_rule() {
            Rule::idty_v10 => Ok(Self::from_versioned_pest_pair(10, idty_vx_pair)?),
            _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "{:#?}",
                idty_vx_pair.as_rule()
            ))),
        }
    }
    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            10 => Ok(IdentityDocument::V10(IdentityDocumentV10::from_pest_pair(
                pair,
            )?)),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum IdentityDocumentStringified {
    V10(IdentityDocumentV10Stringified),
}

impl ToStringObject for IdentityDocument {
    type StringObject = IdentityDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            IdentityDocument::V10(idty) => {
                IdentityDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

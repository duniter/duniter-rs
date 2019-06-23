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

pub mod v10;

pub use v10::{MembershipDocumentV10, MembershipDocumentV10Stringified};

use crate::documents::*;
use crate::text_document_traits::{CompactTextDocument, TextDocument};

/// Wrap an Membership document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum MembershipDocument {
    V10(MembershipDocumentV10),
}

impl Document for MembershipDocument {
    type PublicKey = PubKey;

    #[inline]
    fn version(&self) -> u16 {
        match self {
            MembershipDocument::V10(_) => 10u16,
        }
    }

    #[inline]
    fn currency(&self) -> &str {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.currency(),
        }
    }

    #[inline]
    fn blockstamp(&self) -> Blockstamp {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.blockstamp(),
        }
    }

    #[inline]
    fn issuers(&self) -> &Vec<PubKey> {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.issuers(),
        }
    }

    #[inline]
    fn signatures(&self) -> &Vec<Sig> {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.signatures(),
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.as_bytes(),
        }
    }
}

impl CompactTextDocument for MembershipDocument {
    fn as_compact_text(&self) -> String {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.as_compact_text(),
        }
    }
}

impl TextDocument for MembershipDocument {
    type CompactTextDocument_ = MembershipDocument;

    fn as_text(&self) -> &str {
        match self {
            MembershipDocument::V10(ms_v10) => ms_v10.as_text(),
        }
    }

    fn to_compact_document(&self) -> Self::CompactTextDocument_ {
        match self {
            MembershipDocument::V10(ms_v10) => {
                MembershipDocument::V10(ms_v10.to_compact_document())
            }
        }
    }
}

/// Membership document parser
#[derive(Debug, Clone, Copy)]
pub struct MembershipDocumentParser;

impl TextDocumentParser<Rule> for MembershipDocumentParser {
    type DocumentType = MembershipDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        let mut ms_pairs = DocumentsParser::parse(Rule::membership, doc)?;
        let ms_pair = ms_pairs.next().unwrap(); // get and unwrap the `membership` rule; never fails
        Self::from_pest_pair(ms_pair)
    }
    #[inline]
    fn from_pest_pair(pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let ms_vx_pair = pair.into_inner().next().unwrap(); // get and unwrap the `membership_vX` rule; never fails

        match ms_vx_pair.as_rule() {
            Rule::membership_v10 => Ok(Self::from_versioned_pest_pair(10, ms_vx_pair)?),
            _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "{:#?}",
                ms_vx_pair.as_rule()
            ))),
        }
    }
    #[inline]
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            10 => Ok(MembershipDocument::V10(
                MembershipDocumentV10::from_pest_pair(pair)?,
            )),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MembershipDocumentStringified {
    V10(MembershipDocumentV10Stringified),
}

impl ToStringObject for MembershipDocument {
    type StringObject = MembershipDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            MembershipDocument::V10(idty) => {
                MembershipDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

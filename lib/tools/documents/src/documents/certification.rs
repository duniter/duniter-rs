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

//! Wrappers around Certification documents.

pub mod v10;

pub use v10::{
    CertificationDocumentV10, CertificationDocumentV10Stringified, CompactCertificationDocumentV10,
};

use crate::blockstamp::Blockstamp;
use crate::documents::*;

use dup_crypto::keys::*;
use durs_common_tools::fatal_error;
use pest::Parser;

/// Wrap an Certification document.
///
/// Must be created by parsing a text document or using a builder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum CertificationDocument {
    /// Certification document v10
    V10(CertificationDocumentV10),
}

impl Document for CertificationDocument {
    type PublicKey = PubKey;

    #[inline]
    fn version(&self) -> u16 {
        match self {
            CertificationDocument::V10(_) => 10u16,
        }
    }

    #[inline]
    fn currency(&self) -> &str {
        match self {
            CertificationDocument::V10(cert_v10) => cert_v10.currency(),
        }
    }

    #[inline]
    fn blockstamp(&self) -> Blockstamp {
        match self {
            CertificationDocument::V10(cert_v10) => cert_v10.blockstamp(),
        }
    }

    #[inline]
    fn issuers(&self) -> &Vec<PubKey> {
        match self {
            CertificationDocument::V10(cert_v10) => cert_v10.issuers(),
        }
    }

    #[inline]
    fn signatures(&self) -> &Vec<Sig> {
        match self {
            CertificationDocument::V10(cert_v10) => cert_v10.signatures(),
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            CertificationDocument::V10(cert_v10) => cert_v10.as_bytes(),
        }
    }
}

/// Certification document parser
#[derive(Debug, Clone, Copy)]
pub struct CertificationDocumentParser;

impl TextDocumentParser<Rule> for CertificationDocumentParser {
    type DocumentType = CertificationDocument;

    fn parse(doc: &str) -> Result<Self::DocumentType, TextDocumentParseError> {
        match DocumentsParser::parse(Rule::cert, doc) {
            Ok(mut cert_pairs) => {
                let cert_pair = cert_pairs.next().unwrap(); // get and unwrap the `cert` rule; never fails
                Self::from_pest_pair(cert_pair)
            }
            Err(pest_error) => fatal_error!("{}", pest_error), //Err(TextDocumentParseError::PestError()),
        }
    }
    fn from_pest_pair(cert_pair: Pair<Rule>) -> Result<Self::DocumentType, TextDocumentParseError> {
        let cert_vx_pair = cert_pair.into_inner().next().unwrap(); // get and unwrap the `cert_vX` rule; never fails

        match cert_vx_pair.as_rule() {
            Rule::cert_v10 => Ok(CertificationDocumentParser::from_versioned_pest_pair(
                10,
                cert_vx_pair,
            )?),
            _ => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "{:#?}",
                cert_vx_pair.as_rule()
            ))),
        }
    }
    fn from_versioned_pest_pair(
        version: u16,
        pair: Pair<Rule>,
    ) -> Result<Self::DocumentType, TextDocumentParseError> {
        match version {
            10 => Ok(CertificationDocument::V10(
                CertificationDocumentV10::from_pest_pair(pair)?,
            )),
            v => Err(TextDocumentParseError::UnexpectedVersion(format!(
                "Unsupported version: {}",
                v
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CertificationDocumentStringified {
    V10(CertificationDocumentV10Stringified),
}

impl ToStringObject for CertificationDocument {
    type StringObject = CertificationDocumentStringified;

    fn to_string_object(&self) -> Self::StringObject {
        match self {
            CertificationDocument::V10(idty) => {
                CertificationDocumentStringified::V10(idty.to_string_object())
            }
        }
    }
}

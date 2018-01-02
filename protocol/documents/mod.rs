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

pub mod blockchain10;

use std::fmt::Debug;

/// Common top-level document trait.
///
/// Provide commun methods for any documents of any protocol version.
pub trait Document: Debug {
    /// Get document version.
    fn version(&self) -> u16;

    /// Get document currency.
    fn currency(&self) -> &str;
}

/// Trait allowing access to the document through it's proper protocol version.
///
/// This trait is generic over `VersionEnum` providing all supported protocol version variants.
///
/// A lifetime is specified to allow enum variants to hold references to the document.
pub trait ToProtocolDocument<'a, VersionEnum>: Document {
    /// Get a protocol-specific document wrapped in an enum variant.
    fn associated_protocol(&'a self) -> VersionEnum;
}

/// Trait converting a document to a specialized document wrapped in an enum variant.
///
/// This trait is generic over an `TypeEnum` specific to the protcol version.
///
/// A lifetime is specified to allow enum variants to hold references to the document.
pub trait ToSpecializedDocument<'a, TypeEnum: 'a>: Document {
    /// Get specialized document wrapped in an enum variant.
    fn specialize(&'a self) -> TypeEnum;
}

/// List of blockchain protocol versions.
#[derive(Debug)]
pub enum BlockchainProtocolVersion<'a> {
    /// Version 10.
    V10(&'a ToSpecializedDocument<'a, blockchain10::DocumentType<'a>>),
    /// Version 11. (not done yet, but defined for tests)
    V11(),
}

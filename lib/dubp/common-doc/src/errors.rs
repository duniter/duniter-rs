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

//! Define DUBP Documents errors types.

use dup_crypto::keys::SigError;
use std::collections::HashMap;

/// List of possible errors for document signatures verification.
#[derive(Debug, Eq, PartialEq)]
pub enum DocumentSigsErr {
    /// Not same amount of issuers and signatures.
    /// (issuers count, signatures count)
    IncompletePairs(usize, usize),
    /// Signatures don't match.
    /// List of mismatching pairs indexes.
    Invalid(HashMap<usize, SigError>),
}

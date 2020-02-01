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

//! Implements the Dunitrust Documents Protocol.

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces
)]

#[macro_use]
extern crate failure;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate serde_derive;

pub mod documents;
pub mod parsers;

#[cfg(test)]
mod tests {
    use crate::documents::UserDocumentDUBP;
    use dubp_common_doc::parser::TextDocumentParser;

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

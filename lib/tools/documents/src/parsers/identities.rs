//  Copyright (C) 2018  The Durs Project Developers.
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

use crate::documents::identity::*;
use crate::parsers::*;
use crate::DocumentBuilder;
use dup_crypto::keys::*;

#[derive(Debug, Fail)]
#[fail(display = "Fail to parse identity : {:?} !", cause)]
pub struct ParseIdentityError {
    pub cause: String,
}

/// Parse a compact identity
pub fn parse_compact_identities(
    currency: &str,
    str_identities: Vec<&str>,
) -> Result<Vec<IdentityDocument>, ParseIdentityError> {
    let mut identities = Vec::with_capacity(str_identities.len());

    for str_identity in str_identities {
        let idty_elements: Vec<&str> = str_identity.split(':').collect();
        let issuer = match ed25519::PublicKey::from_base58(idty_elements[0]) {
            Ok(pubkey) => PubKey::Ed25519(pubkey),
            Err(_) => {
                return Err(ParseIdentityError {
                    cause: "invalid pubkey".to_owned(),
                });
            }
        };
        let signature = match ed25519::Signature::from_base64(idty_elements[1]) {
            Ok(sig) => Sig::Ed25519(sig),
            Err(_) => {
                return Err(ParseIdentityError {
                    cause: "invalid signature".to_owned(),
                });
            }
        };
        let blockstamp = match Blockstamp::from_string(idty_elements[2]) {
            Ok(blockstamp) => blockstamp,
            Err(_) => {
                return Err(ParseIdentityError {
                    cause: "invalid blockstamp".to_owned(),
                });
            }
        };
        let username = idty_elements[3];
        let idty_doc_builder = IdentityDocumentBuilder {
            currency,
            username,
            blockstamp: &blockstamp,
            issuer: &issuer,
        };
        identities.push(idty_doc_builder.build_with_signature(vec![signature]))
    }

    Ok(identities)
}

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

use crate::documents::revocation::{CompactRevocationDocumentV10, RevocationDocumentV10};
use crate::text_document_traits::TextDocumentFormat;
use dup_crypto::keys::*;

/// Parse array of revocations json documents into vector of `CompactRevocationDocumentV10`
pub fn parse_revocations_into_compact(
    str_revocations: &[&str],
) -> Vec<TextDocumentFormat<RevocationDocumentV10>> {
    let mut revocations: Vec<TextDocumentFormat<RevocationDocumentV10>> = Vec::new();
    for revocation in str_revocations {
        let revocations_datas: Vec<&str> = revocation.split(':').collect();
        if revocations_datas.len() == 2 {
            revocations.push(TextDocumentFormat::Compact(CompactRevocationDocumentV10 {
                issuer: PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(revocations_datas[0])
                        .expect("Receive block in wrong format !"),
                ),
                signature: Sig::Ed25519(
                    ed25519::Signature::from_base64(revocations_datas[1])
                        .expect("Receive block in wrong format !"),
                ),
            }));
        }
    }
    revocations
}

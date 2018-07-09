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

extern crate serde_json;

use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::revocation::CompactRevocationDocument;
use duniter_documents::blockchain::v10::documents::{RevocationDocument, TextDocumentFormat};

/// Parse array of revocations json documents into vector of `CompactRevocationDocument`
pub fn parse_revocations_into_compact(
    json_recocations: &[serde_json::Value],
) -> Vec<TextDocumentFormat<RevocationDocument>> {
    let mut revocations: Vec<TextDocumentFormat<RevocationDocument>> = Vec::new();
    for revocation in json_recocations.iter() {
        let revocations_datas: Vec<&str> = revocation
            .as_str()
            .expect("Receive block in wrong format !")
            .split(':')
            .collect();
        if revocations_datas.len() == 2 {
            revocations.push(TextDocumentFormat::Compact(CompactRevocationDocument {
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

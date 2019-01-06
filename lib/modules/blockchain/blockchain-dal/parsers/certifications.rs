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

use dubp_documents::documents::certification::{
    CertificationDocument, CompactCertificationDocument,
};
use dubp_documents::text_document_traits::TextDocumentFormat;
use dubp_documents::BlockId;
use dup_crypto::keys::*;
use serde_json;

/// Parse array of certification json documents into vector of `CompactCertificationDocument`
pub fn parse_certifications_into_compact(
    json_certs: &[serde_json::Value],
) -> Vec<TextDocumentFormat<CertificationDocument>> {
    let mut certifications: Vec<TextDocumentFormat<CertificationDocument>> = Vec::new();
    for certification in json_certs.iter() {
        let certifications_datas: Vec<&str> = certification
            .as_str()
            .expect("Receive block in wrong format : fail to split cert !")
            .split(':')
            .collect();
        if certifications_datas.len() == 4 {
            certifications.push(TextDocumentFormat::Compact(CompactCertificationDocument {
                issuer: PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(certifications_datas[0])
                        .expect("Receive block in wrong format : fail to parse issuer !"),
                ),
                target: PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(certifications_datas[1])
                        .expect("Receive block in wrong format : fail to parse target !"),
                ),
                block_number: BlockId(
                    certifications_datas[2]
                        .parse()
                        .expect("Receive block in wrong format : fail to parse block number !"),
                ),
                signature: Sig::Ed25519(
                    ed25519::Signature::from_base64(certifications_datas[3])
                        .expect("Receive block in wrong format : fail to parse signature !"),
                ),
            }));
        }
    }
    certifications
}

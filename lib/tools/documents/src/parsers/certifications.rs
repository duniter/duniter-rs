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

use crate::documents::certification::{CertificationDocumentV10, CompactCertificationDocumentV10};
use crate::text_document_traits::TextDocumentFormat;
use crate::BlockNumber;
use dup_crypto::keys::*;

/// Parse array of certification json documents into vector of `CompactCertificationDocument`
pub fn parse_certifications_into_compact(
    str_certs: &[&str],
) -> Vec<TextDocumentFormat<CertificationDocumentV10>> {
    let mut certifications: Vec<TextDocumentFormat<CertificationDocumentV10>> = Vec::new();
    for certification in str_certs {
        let certifications_datas: Vec<&str> = certification.split(':').collect();
        if certifications_datas.len() == 4 {
            certifications.push(TextDocumentFormat::Compact(
                CompactCertificationDocumentV10 {
                    issuer: PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(certifications_datas[0])
                            .expect("Receive block in wrong format : fail to parse issuer !"),
                    ),
                    target: PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(certifications_datas[1])
                            .expect("Receive block in wrong format : fail to parse target !"),
                    ),
                    block_number: BlockNumber(
                        certifications_datas[2]
                            .parse()
                            .expect("Receive block in wrong format : fail to parse block number !"),
                    ),
                    signature: Sig::Ed25519(
                        ed25519::Signature::from_base64(certifications_datas[3])
                            .expect("Receive block in wrong format : fail to parse signature !"),
                    ),
                },
            ));
        }
    }
    certifications
}

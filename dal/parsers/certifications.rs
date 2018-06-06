extern crate serde;
extern crate serde_json;

use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::certification::CompactCertificationDocument;
use duniter_documents::blockchain::v10::documents::{CertificationDocument, TextDocumentFormat};
use duniter_documents::BlockId;

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

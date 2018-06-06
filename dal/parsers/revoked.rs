extern crate serde_json;

use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::revocation::CompactRevocationDocument;
use duniter_documents::blockchain::v10::documents::{RevocationDocument, TextDocumentFormat};

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

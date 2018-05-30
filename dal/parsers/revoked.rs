extern crate serde_json;

use duniter_crypto::keys::{ed25519, PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::revocation::{
    CompactRevocationDocument, RevocationDocumentBuilder,
};
use duniter_documents::blockchain::v10::documents::{
    IdentityDocument, RevocationDocument, TextDocumentFormat,
};
use duniter_documents::blockchain::{Document, DocumentBuilder};

use super::super::identity::DALIdentity;
use super::super::DuniterDB;

use std::collections::HashMap;

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
                issuer: PublicKey::from_base58(revocations_datas[0])
                    .expect("Receive block in wrong format !"),
                signature: Signature::from_base64(revocations_datas[1])
                    .expect("Receive block in wrong format !"),
            }));
        }
    }
    revocations
}

pub fn parse_revocations(
    currency: &str,
    db: &DuniterDB,
    block_identities: &HashMap<ed25519::PublicKey, IdentityDocument>,
    json_datas: &str,
) -> Option<Vec<TextDocumentFormat<RevocationDocument>>> {
    let raw_revocations: serde_json::Value = serde_json::from_str(json_datas).unwrap();

    if raw_revocations.is_array() {
        Some(parse_revocations_from_json_value(
            currency,
            db,
            block_identities,
            raw_revocations.as_array().unwrap(),
        ))
    } else {
        None
    }
}

pub fn parse_revocations_from_json_value(
    currency: &str,
    db: &DuniterDB,
    block_identities: &HashMap<ed25519::PublicKey, IdentityDocument>,
    array_revocations: &[serde_json::Value],
) -> Vec<TextDocumentFormat<RevocationDocument>> {
    let mut revocations: Vec<TextDocumentFormat<RevocationDocument>> = Vec::new();
    for revocation in array_revocations.iter() {
        let revocations_datas: Vec<&str> = revocation.as_str().unwrap().split(':').collect();
        if revocations_datas.len() == 2 {
            let idty_pubkey: ed25519::PublicKey =
                PublicKey::from_base58(revocations_datas[0]).unwrap();
            let idty_doc: IdentityDocument = match block_identities.get(&idty_pubkey) {
                Some(idty_doc) => idty_doc.clone(),
                None => {
                    let dal_idty = DALIdentity::get_identity(currency, db, &idty_pubkey).unwrap();
                    dal_idty.idty_doc
                }
            };
            let revoc_doc_builder = RevocationDocumentBuilder {
                currency,
                issuer: &idty_pubkey,
                identity_username: idty_doc.username(),
                identity_blockstamp: &idty_doc.blockstamp(),
                identity_sig: &idty_doc.signatures()[0],
            };
            let revoc_sig = Signature::from_base64(revocations_datas[1]).unwrap();
            revocations.push(TextDocumentFormat::Complete(
                revoc_doc_builder.build_with_signature(vec![revoc_sig]),
            ));
        }
    }
    revocations
}

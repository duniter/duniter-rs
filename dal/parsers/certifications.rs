extern crate serde;
extern crate serde_json;
extern crate sqlite;

use super::super::block::DALBlock;
use super::super::identity::DALIdentity;
use super::super::DuniterDB;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::certification::{
    CertificationDocumentBuilder, CompactCertificationDocument,
};
use duniter_documents::blockchain::v10::documents::{
    CertificationDocument, IdentityDocument, TextDocumentFormat,
};
use duniter_documents::blockchain::{Document, DocumentBuilder};
use duniter_documents::{BlockHash, BlockId, Blockstamp, Hash};
use std::collections::HashMap;

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

pub fn parse_certifications_from_json_value(
    currency: &str,
    db: &DuniterDB,
    block_identities: &HashMap<PubKey, IdentityDocument>,
    array_certifications: &[serde_json::Value],
) -> Vec<TextDocumentFormat<CertificationDocument>> {
    let mut certifications: Vec<TextDocumentFormat<CertificationDocument>> = Vec::new();
    for certification in array_certifications.iter() {
        let certification_datas: Vec<&str> = certification
            .as_str()
            .expect("Fail to parse certs : json isn't str !")
            .split(':')
            .collect();
        if certification_datas.len() == 4 {
            let target = PubKey::Ed25519(
                ed25519::PublicKey::from_base58(certification_datas[1])
                    .expect("Fail to parse cert target !"),
            );
            let target_idty_doc: IdentityDocument = match block_identities.get(&target) {
                Some(idty_doc) => idty_doc.clone(),
                None => {
                    let dal_idty = DALIdentity::get_identity(currency, db, &target)
                        .expect("target identity not found in bdd !");
                    dal_idty.idty_doc
                }
            };
            let cert_blockstamp_id = BlockId(
                certification_datas[2]
                    .parse()
                    .expect("Fail to parse cert blockstamp !"),
            );
            let cert_builder =
                CertificationDocumentBuilder {
                    currency,
                    issuer: &PubKey::Ed25519(
                        ed25519::PublicKey::from_base58(certification_datas[0])
                            .expect("Fail to parse cert issuer !"),
                    ),
                    blockstamp: &Blockstamp {
                        id: cert_blockstamp_id,
                        hash: if cert_blockstamp_id == BlockId(0) {
                            BlockHash(Hash::from_hex(
                            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                        ).expect("Fail to parse cert : invalid genesis hash"))
                        } else {
                            DALBlock::get_block_hash(db, &cert_blockstamp_id).expect(&format!(
                                "Fatal Error : Block {} not found in bdd !",
                                cert_blockstamp_id
                            ))
                        },
                    },
                    target: &target,
                    identity_username: target_idty_doc.username(),
                    identity_blockstamp: &target_idty_doc.blockstamp(),
                    identity_sig: &target_idty_doc.signatures()[0],
                };
            let cert_sig = Sig::Ed25519(
                ed25519::Signature::from_base64(certification_datas[3])
                    .expect("Fail to parse cert sig !"),
            );
            certifications.push(TextDocumentFormat::Complete(
                cert_builder.build_with_signature(vec![cert_sig]),
            ));
        }
    }
    certifications
}

pub fn parse_certifications(
    currency: &str,
    db: &DuniterDB,
    block_identities: &HashMap<PubKey, IdentityDocument>,
    json_datas: &str,
) -> Option<Vec<TextDocumentFormat<CertificationDocument>>> {
    let raw_certifications: serde_json::Value =
        serde_json::from_str(json_datas).expect("Fail to parse certs: str isn't json !");

    if raw_certifications.is_array() {
        Some(parse_certifications_from_json_value(
            currency,
            db,
            block_identities,
            raw_certifications
                .as_array()
                .expect("Fail to parse certs: json datas must be an array !"),
        ))
    } else {
        None
    }
}

extern crate serde;
extern crate serde_json;
extern crate sqlite;

use super::super::block::DALBlock;
use super::super::identity::DALIdentity;
use super::super::DuniterDB;
use duniter_crypto::keys::{ed25519, PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::certification::CertificationDocumentBuilder;
use duniter_documents::blockchain::v10::documents::{CertificationDocument, IdentityDocument};
use duniter_documents::blockchain::{Document, DocumentBuilder};
use duniter_documents::{BlockHash, BlockId, Blockstamp, Hash};
use duniter_wotb::NodeId;
use std::collections::HashMap;

pub fn parse_certifications_from_json_value(
    currency: &str,
    db: &DuniterDB,
    wotb_index: &HashMap<ed25519::PublicKey, NodeId>,
    block_identities: &HashMap<ed25519::PublicKey, IdentityDocument>,
    array_certifications: &[serde_json::Value],
) -> Vec<CertificationDocument> {
    let mut certifications: Vec<CertificationDocument> = Vec::new();
    for certification in array_certifications.iter() {
        let certification_datas: Vec<&str> = certification.as_str().unwrap().split(':').collect();
        if certification_datas.len() == 4 {
            let target = PublicKey::from_base58(certification_datas[1])
                .expect("Fail to parse cert target !");
            let target_idty_doc: IdentityDocument = match block_identities.get(&target) {
                Some(idty_doc) => idty_doc.clone(),
                None => {
                    let target_wotb_id = wotb_index.get(&target).expect(&format!(
                        "target identity {} not found in wotb index !",
                        target.to_string()
                    ));
                    let dal_idty = DALIdentity::get_identity(currency, db, target_wotb_id)
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
                    issuer: &PublicKey::from_base58(certification_datas[0])
                        .expect("Fail to parse cert issuer !"),
                    blockstamp: &Blockstamp {
                        id: cert_blockstamp_id,
                        hash: if cert_blockstamp_id == BlockId(0) {
                            BlockHash(Hash::from_hex(
                            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
                        ).unwrap())
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
            let cert_sig =
                Signature::from_base64(certification_datas[3]).expect("Fail to parse cert sig !");
            certifications.push(cert_builder.build_with_signature(vec![cert_sig]));
        }
    }
    certifications
}

pub fn parse_certifications(
    currency: &str,
    db: &DuniterDB,
    wotb_index: &HashMap<ed25519::PublicKey, NodeId>,
    block_identities: &HashMap<ed25519::PublicKey, IdentityDocument>,
    json_datas: &str,
) -> Option<Vec<CertificationDocument>> {
    let raw_certifications: serde_json::Value = serde_json::from_str(json_datas).unwrap();

    if raw_certifications.is_array() {
        Some(parse_certifications_from_json_value(
            currency,
            db,
            wotb_index,
            block_identities,
            raw_certifications.as_array().unwrap(),
        ))
    } else {
        None
    }
}

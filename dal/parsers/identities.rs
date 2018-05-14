extern crate serde_json;
extern crate sqlite;

use duniter_crypto::keys::{PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::identity::IdentityDocumentBuilder;
use duniter_documents::blockchain::v10::documents::IdentityDocument;
use duniter_documents::blockchain::DocumentBuilder;
use duniter_documents::Blockstamp;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdentityParseError {
    WrongFormat(),
}

pub fn parse_identities(currency: &str, json_datas: &str) -> Option<Vec<IdentityDocument>> {
    let raw_idties: serde_json::Value = serde_json::from_str(json_datas).unwrap();
    if raw_idties.is_array() {
        return Some(
            parse_identities_from_json_value(currency, raw_idties.as_array().unwrap())
                .iter()
                .map(|i| {
                    i.clone()
                        .expect("Fatal error : Fail to parse identity from local DB !")
                })
                .collect(),
        );
    }
    None
}

pub fn parse_identities_from_json_value(
    currency: &str,
    array_identities: &[serde_json::Value],
) -> Vec<Result<IdentityDocument, IdentityParseError>> {
    array_identities
        .iter()
        .map(|idty| {
            let idty_datas: Vec<&str> = idty.as_str().unwrap().split(':').collect();
            if idty_datas.len() == 4 {
                let idty_doc_builder = IdentityDocumentBuilder {
                    currency,
                    issuer: &PublicKey::from_base58(idty_datas[0]).unwrap(),
                    blockstamp: &Blockstamp::from_string(idty_datas[2]).unwrap(),
                    username: idty_datas[3],
                };
                let idty_sig = Signature::from_base64(idty_datas[1]).unwrap();
                //memberships.push(membership_doc_builder.build_with_signature(vec![membership_sig]));
                Ok(idty_doc_builder.build_with_signature(vec![idty_sig]))
            } else {
                Err(IdentityParseError::WrongFormat())
            }
        })
        .collect()

    /*for membership in array_memberships.iter() {
        let membership_datas: Vec<&str> = membership.as_str().unwrap().split(':').collect();
        if membership_datas.len() == 5 {
            let membership_doc_builder = IdentityDocumentBuilder {
                currency,
                issuer: &PublicKey::from_base58(membership_datas[0]).unwrap(),
                blockstamp: &Blockstamp::from_string(membership_datas[2]).unwrap(),
                membership: membership_type,
                identity_username: membership_datas[4],
                identity_blockstamp: &Blockstamp::from_string(membership_datas[3]).unwrap(),
            };
            let membership_sig = Signature::from_base64(membership_datas[1]).unwrap();
            memberships.push(membership_doc_builder.build_with_signature(vec![membership_sig]));
        }
    }
    memberships*/
}

pub fn parse_compact_identity(
    currency: &str,
    source: &serde_json::Value,
) -> Option<IdentityDocument> {
    if source.is_string() {
        let idty_elements: Vec<&str> = source.as_str().unwrap().split(':').collect();
        let issuer = match PublicKey::from_base58(idty_elements[0]) {
            Ok(pubkey) => pubkey,
            Err(_) => return None,
        };
        let signature = match Signature::from_base64(idty_elements[1]) {
            Ok(sig) => sig,
            Err(_) => return None,
        };
        let blockstamp = match Blockstamp::from_string(idty_elements[2]) {
            Ok(blockstamp) => blockstamp,
            Err(_) => return None,
        };
        let username = idty_elements[3];
        let idty_doc_builder = IdentityDocumentBuilder {
            currency,
            username,
            blockstamp: &blockstamp,
            issuer: &issuer,
        };
        Some(idty_doc_builder.build_with_signature(vec![signature]))
    } else {
        None
    }
}

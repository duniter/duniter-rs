use dubp_documents::documents::identity::*;
use dubp_documents::Blockstamp;
use dubp_documents::DocumentBuilder;
use dup_crypto::keys::*;
use unwrap::unwrap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdentityParseError {
    WrongFormat(),
}

pub fn parse_identities(currency: &str, json_datas: &str) -> Option<Vec<IdentityDocument>> {
    let raw_idties: serde_json::Value = unwrap!(serde_json::from_str(json_datas));
    if raw_idties.is_array() {
        return Some(
            parse_identities_from_json_value(currency, unwrap!(raw_idties.as_array()))
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
            let idty_datas: Vec<&str> = unwrap!(idty.as_str()).split(':').collect();
            if idty_datas.len() == 4 {
                let idty_doc_builder = IdentityDocumentBuilder {
                    currency,
                    issuer: &PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        idty_datas[0]
                    ))),
                    blockstamp: &unwrap!(Blockstamp::from_string(idty_datas[2])),
                    username: idty_datas[3],
                };
                let idty_sig =
                    Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(idty_datas[1])));
                //memberships.push(membership_doc_builder.build_with_signature(vec![membership_sig]));
                Ok(idty_doc_builder.build_with_signature(vec![idty_sig]))
            } else {
                Err(IdentityParseError::WrongFormat())
            }
        })
        .collect()
}

pub fn parse_compact_identity(
    currency: &str,
    source: &serde_json::Value,
) -> Option<IdentityDocument> {
    if source.is_string() {
        let idty_elements: Vec<&str> = unwrap!(source.as_str()).split(':').collect();
        let issuer = match ed25519::PublicKey::from_base58(idty_elements[0]) {
            Ok(pubkey) => PubKey::Ed25519(pubkey),
            Err(_) => return None,
        };
        let signature = match ed25519::Signature::from_base64(idty_elements[1]) {
            Ok(sig) => Sig::Ed25519(sig),
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

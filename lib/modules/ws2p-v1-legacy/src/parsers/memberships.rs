use dubp_documents::documents::membership::*;
use dubp_documents::Blockstamp;
use dubp_documents::DocumentBuilder;
use dup_crypto::keys::*;
use unwrap::unwrap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MembershipParseError {
    WrongFormat(),
}

pub fn parse_memberships(
    currency: &str,
    membership_type: MembershipType,
    json_datas: &str,
) -> Option<Vec<MembershipDocument>> {
    let raw_memberships: serde_json::Value = unwrap!(serde_json::from_str(json_datas));
    if raw_memberships.is_array() {
        return Some(
            parse_memberships_from_json_value(
                currency,
                membership_type,
                unwrap!(raw_memberships.as_array()),
            )
            .iter()
            .map(|m| {
                m.clone()
                    .expect("Fatal error : Fail to parse membership from local DB !")
            })
            .collect(),
        );
    }
    None
}

pub fn parse_memberships_from_json_value(
    currency: &str,
    membership_type: MembershipType,
    array_memberships: &[serde_json::Value],
) -> Vec<Result<MembershipDocument, MembershipParseError>> {
    //let memberships: Vec<MembershipDocument> = Vec::new();
    array_memberships
        .iter()
        .map(|membership| {
            let membership_datas: Vec<&str> = unwrap!(membership.as_str()).split(':').collect();
            if membership_datas.len() == 5 {
                let membership_doc_builder = MembershipDocumentBuilder {
                    currency,
                    issuer: &PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
                        membership_datas[0]
                    ))),
                    blockstamp: &unwrap!(Blockstamp::from_string(membership_datas[2])),
                    membership: membership_type,
                    identity_username: membership_datas[4],
                    identity_blockstamp: &unwrap!(Blockstamp::from_string(membership_datas[3])),
                };
                let membership_sig = Sig::Ed25519(unwrap!(ed25519::Signature::from_base64(
                    membership_datas[1]
                )));
                Ok(membership_doc_builder.build_with_signature(vec![membership_sig]))
            } else {
                Err(MembershipParseError::WrongFormat())
            }
        })
        .collect()
}

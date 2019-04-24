use dup_crypto::keys::ed25519;
use dup_crypto::keys::*;
use unwrap::unwrap;

pub fn parse_exclusions(json_datas: &str) -> Option<Vec<PubKey>> {
    let raw_exclusions: serde_json::Value = unwrap!(serde_json::from_str(json_datas));

    if raw_exclusions.is_array() {
        Some(parse_exclusions_from_json_value(unwrap!(
            raw_exclusions.as_array()
        )))
    } else {
        None
    }
}

pub fn parse_exclusions_from_json_value(array_exclusions: &[serde_json::Value]) -> Vec<PubKey> {
    let mut exclusions: Vec<PubKey> = Vec::new();
    for exclusion in array_exclusions.iter() {
        exclusions.push(PubKey::Ed25519(unwrap!(ed25519::PublicKey::from_base58(
            unwrap!(exclusion.as_str())
        ))));
    }
    exclusions
}

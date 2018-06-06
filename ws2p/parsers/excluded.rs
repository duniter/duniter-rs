extern crate serde;
extern crate serde_json;

use duniter_crypto::keys::ed25519;
use duniter_crypto::keys::*;

pub fn parse_exclusions(json_datas: &str) -> Option<Vec<PubKey>> {
    let raw_exclusions: serde_json::Value = serde_json::from_str(json_datas).unwrap();

    if raw_exclusions.is_array() {
        Some(parse_exclusions_from_json_value(
            raw_exclusions.as_array().unwrap(),
        ))
    } else {
        None
    }
}

pub fn parse_exclusions_from_json_value(array_exclusions: &[serde_json::Value]) -> Vec<PubKey> {
    let mut exclusions: Vec<PubKey> = Vec::new();
    for exclusion in array_exclusions.iter() {
        exclusions.push(PubKey::Ed25519(
            ed25519::PublicKey::from_base58(exclusion.as_str().unwrap()).unwrap(),
        ));
    }
    exclusions
}

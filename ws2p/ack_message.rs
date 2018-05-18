extern crate duniter_crypto;
extern crate serde;
extern crate serde_json;

use self::serde::ser::{Serialize, SerializeStruct, Serializer};
use super::WS2PMessage;
use duniter_crypto::keys::ed25519::PublicKey as ed25519PublicKey;
use duniter_crypto::keys::PublicKey;

#[derive(Debug, Clone)]
pub struct WS2PAckMessageV1 {
    pub currency: String,
    pub pubkey: ed25519PublicKey,
    pub challenge: String,
    pub signature: Option<duniter_crypto::keys::ed25519::Signature>,
}

impl WS2PMessage for WS2PAckMessageV1 {
    fn parse(v: &serde_json::Value, currency: String) -> Option<Self> {
        let pubkey = match v.get("pub") {
            Some(pubkey) => pubkey.as_str().unwrap().to_string(),
            None => return None,
        };
        let signature = match v.get("sig") {
            Some(signature) => signature.as_str().unwrap().to_string(),
            None => return None,
        };
        let pubkey: ed25519PublicKey = ed25519PublicKey::from_base58(&pubkey).unwrap();
        let signature: Option<duniter_crypto::keys::ed25519::Signature> =
            Some(duniter_crypto::keys::Signature::from_base64(&signature).unwrap());
        Some(WS2PAckMessageV1 {
            currency,
            pubkey,
            challenge: "".to_string(),
            signature,
        })
    }
    fn to_raw(&self) -> String {
        format!(
            "WS2P:ACK:{}:{}:{}",
            self.currency, self.pubkey, self.challenge
        )
    }
    fn verify(&self) -> bool {
        self.pubkey
            .verify(self.to_raw().as_bytes(), &self.signature.unwrap())
    }
}

impl Serialize for WS2PAckMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut connect_message_in_json = serializer.serialize_struct("message", 3)?;
        connect_message_in_json.serialize_field("auth", "ACK")?;
        connect_message_in_json.serialize_field("pub", &self.pubkey.to_string())?;
        connect_message_in_json.serialize_field(
            "sig",
            &self
                .signature
                .expect("Fail to serialize ACK message : the signature field is set to None !")
                .to_string(),
        )?;
        connect_message_in_json.end()
    }
}

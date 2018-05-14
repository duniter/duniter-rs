extern crate duniter_crypto;
extern crate serde;
extern crate serde_json;

use self::serde::ser::{Serialize, SerializeStruct, Serializer};
use super::WS2PMessage;
use duniter_crypto::keys::ed25519::PublicKey as ed25519PublicKey;
use duniter_crypto::keys::PublicKey;

#[derive(Debug, Clone)]
pub struct WS2POkMessageV1 {
    pub currency: String,
    pub pubkey: ed25519PublicKey,
    pub challenge: String,
    pub signature: Option<duniter_crypto::keys::ed25519::Signature>,
}

impl WS2PMessage for WS2POkMessageV1 {
    fn parse(v: &serde_json::Value, currency: String) -> Option<Self> {
        let signature = match v.get("sig") {
            Some(signature) => signature
                .as_str()
                .expect("Parsing of OK message : fail to convert sig to str")
                .to_string(),
            None => return None,
        };
        let pubkey: ed25519PublicKey = ed25519PublicKey::from_base58(
            "969qRJs8KhsnkyzqarpL4RKZGMdVKNbZgu8fhsigM7Lj",
        ).expect("fail to create default pubkey !");
        let signature: Option<duniter_crypto::keys::ed25519::Signature> = Some(
            duniter_crypto::keys::Signature::from_base64(&signature)
                .expect("fail to parse signature of OK message !"),
        );
        Some(WS2POkMessageV1 {
            currency,
            pubkey,
            challenge: "".to_string(),
            signature,
        })
    }
    fn to_raw(&self) -> String {
        format!(
            "WS2P:OK:{}:{}:{}",
            self.currency, self.pubkey, self.challenge
        )
    }
    fn verify(&self) -> bool {
        self.pubkey
            .verify(self.to_raw().as_bytes(), &self.signature.unwrap())
    }
}

impl Serialize for WS2POkMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut connect_message_in_json = serializer.serialize_struct("message", 2)?;
        connect_message_in_json.serialize_field("auth", "OK")?;
        connect_message_in_json.serialize_field(
            "sig",
            &self
                .signature
                .expect("Fail to serialize OK message : the signature field is set to None !")
                .to_string(),
        )?;
        connect_message_in_json.end()
    }
}

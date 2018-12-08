use super::WS2PMessage;
use dup_crypto::keys::*;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, Clone)]
pub struct WS2PConnectMessageV1 {
    pub currency: String,
    pub pubkey: PubKey,
    pub challenge: String,
    pub signature: Option<Sig>,
}

impl WS2PMessage for WS2PConnectMessageV1 {
    fn parse(v: &serde_json::Value, currency: String) -> Option<Self> {
        let pubkey = match v.get("pub") {
            Some(pubkey) => pubkey.as_str().unwrap().to_string(),
            None => return None,
        };
        let challenge = match v.get("challenge") {
            Some(challenge) => challenge.as_str().unwrap().to_string(),
            None => return None,
        };
        let signature = match v.get("sig") {
            Some(signature) => signature.as_str().unwrap().to_string(),
            None => return None,
        };
        let pubkey = PubKey::Ed25519(ed25519::PublicKey::from_base58(&pubkey).unwrap());
        let signature = Some(Sig::Ed25519(
            ed25519::Signature::from_base64(&signature).unwrap(),
        ));
        Some(WS2PConnectMessageV1 {
            currency,
            pubkey,
            challenge,
            signature,
        })
    }
    fn to_raw(&self) -> String {
        format!(
            "WS2P:CONNECT:{}:{}:{}",
            self.currency, self.pubkey, self.challenge
        )
    }
    fn verify(&self) -> bool {
        self.pubkey
            .verify(self.to_raw().as_bytes(), &self.signature.unwrap())
    }
}

impl Serialize for WS2PConnectMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut connect_message_in_json = serializer.serialize_struct("message", 4)?;
        connect_message_in_json.serialize_field("auth", "CONNECT")?;
        connect_message_in_json.serialize_field("pub", &self.pubkey.to_string())?;
        connect_message_in_json.serialize_field("challenge", &self.challenge)?;
        connect_message_in_json.serialize_field(
            "sig",
            &self
                .signature
                .expect("Fail to serialize CONNECT message : the signature field is set to None !")
                .to_string(),
        )?;
        connect_message_in_json.end()
    }
}

use crate::*;
use dup_crypto::keys::*;
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, Clone)]
pub struct WS2PAckMessageV1 {
    pub currency: String,
    pub pubkey: PubKey,
    pub challenge: String,
    pub signature: Option<Sig>,
}

impl WS2PMessage for WS2PAckMessageV1 {
    fn parse(v: &serde_json::Value, currency: String) -> Result<Self, WS2PMsgParseErr> {
        let pubkey = match v.get("pub") {
            Some(pubkey) => pubkey.as_str().ok_or(WS2PMsgParseErr {})?.to_string(),
            None => return Err(WS2PMsgParseErr {}),
        };
        let signature = match v.get("sig") {
            Some(signature) => signature.as_str().ok_or(WS2PMsgParseErr {})?.to_string(),
            None => return Err(WS2PMsgParseErr {}),
        };
        let pubkey = PubKey::Ed25519(ed25519::PublicKey::from_base58(&pubkey)?);
        let signature: Option<Sig> =
            Some(Sig::Ed25519(ed25519::Signature::from_base64(&signature)?));
        Ok(WS2PAckMessageV1 {
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
        if let Some(sig) = self.signature {
            self.pubkey.verify(self.to_raw().as_bytes(), &sig).is_ok()
        } else {
            false
        }
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

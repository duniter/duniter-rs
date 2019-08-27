//  Copyright (C) 2017-2019  The AXIOM TEAM Association.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Generic code for signing data in text format

use super::*;

/// Signatureable in text format
pub trait TextSignable: Debug + Clone {
    /// Return signable text
    fn as_signable_text(&self) -> String;
    /// Return entity issuer pubkey
    fn issuer_pubkey(&self) -> PubKey;
    /// Return entity signature
    fn signature(&self) -> Option<Sig>;
    /// Change signature
    fn set_signature(&mut self, _signature: Sig);
    /// Sign entity
    fn sign(&mut self, signator: &SignatorEnum) -> Result<String, SignError> {
        if self.signature().is_some() {
            return Err(SignError::AlreadySign);
        }
        match self.issuer_pubkey() {
            PubKey::Ed25519(_) => match signator {
                SignatorEnum::Ed25519(ed25519_signator) => {
                    let text = self.as_signable_text();
                    let sig = ed25519_signator.sign(&text.as_bytes());
                    self.set_signature(Sig::Ed25519(sig));
                    let str_sig = sig.to_base64();
                    Ok(format!("{}{}", text, str_sig))
                }
                _ => Err(SignError::WrongAlgo),
            },
            _ => Err(SignError::WrongAlgo),
        }
    }
    /// Check signature of entity
    fn verify(&self) -> Result<(), SigError> {
        if let Some(signature) = self.signature() {
            match self.issuer_pubkey() {
                PubKey::Ed25519(pubkey) => match signature {
                    Sig::Ed25519(sig) => {
                        pubkey.verify(&self.as_signable_text().as_bytes(), &sig)
                        /*
                        if pubkey.verify(&self.as_signable_text().as_bytes(), &sig) {
                            Ok(())
                        } else {
                            Err(SigError::InvalidSig())
                        }
                        */
                    }
                    _ => Err(SigError::NotSameAlgo),
                },
                _ => Err(SigError::NotSameAlgo),
            }
        } else {
            Err(SigError::NotSig)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Debug, Clone)]
    struct TextSignableTestImpl {
        issuer: PubKey,
        text: String,
        sig: Option<Sig>,
    }

    impl TextSignable for TextSignableTestImpl {
        fn as_signable_text(&self) -> String {
            format!("{}:{}", self.issuer, self.text)
        }
        fn issuer_pubkey(&self) -> PubKey {
            self.issuer
        }
        fn signature(&self) -> Option<Sig> {
            self.sig
        }
        fn set_signature(&mut self, new_signature: Sig) {
            self.sig = Some(new_signature);
        }
    }

    #[test]
    fn test_text_signable() {
        let key_pair = super::super::tests::valid_key_pair_1();

        let signator = key_pair
            .generate_signator()
            .expect("fail to generate signator");

        let mut text_signable = TextSignableTestImpl {
            issuer: key_pair.public_key(),
            text: "toto".to_owned(),
            sig: None,
        };

        assert_eq!(Err(SigError::NotSig), text_signable.verify());
        assert_eq!(
            Err(SignError::WrongAlgo),
            text_signable.sign(&SignatorEnum::Schnorr())
        );
        text_signable.issuer = PubKey::Schnorr();
        assert_eq!(Err(SignError::WrongAlgo), text_signable.sign(&signator));
        text_signable.issuer = key_pair.public_key();
        assert_eq!(
            Ok("VYgskcKKh525MzFRzpCiT5KXCQrnFLTnzMLffbvm9uw:toto+IC1fFkkYo5ox2loc1IMLCtrir1i6oyljfshNXIyXVcz6sJMFqn+6o8Zip4XdTzoBEORkbcnEnqQEr4TgaHpCw==".to_owned()),
            text_signable.sign(&signator)
        );
        assert_eq!(Err(SignError::AlreadySign), text_signable.sign(&signator));
        assert_eq!(Ok(()), text_signable.verify());
        let old_sig = text_signable.sig.replace(Sig::Schnorr());
        assert_eq!(Err(SigError::NotSameAlgo), text_signable.verify());
        text_signable.sig = old_sig;
        text_signable.issuer = PubKey::Schnorr();
        assert_eq!(Err(SigError::NotSameAlgo), text_signable.verify());
    }
}

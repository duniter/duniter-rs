//  Copyright (C) 2018  The Dunitrust Project Developers.
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
    fn sign(&mut self, priv_key: PrivKey) -> Result<String, SignError> {
        if self.signature().is_some() {
            return Err(SignError::AlreadySign());
        }
        let text = self.as_signable_text();
        match self.issuer_pubkey() {
            PubKey::Ed25519(_) => match priv_key {
                PrivKey::Ed25519(priv_key) => {
                    let sig = priv_key.sign(&text.as_bytes());
                    self.set_signature(Sig::Ed25519(sig));
                    let str_sig = sig.to_base64();
                    Ok(format!("{}{}", text, str_sig))
                }
                _ => Err(SignError::WrongAlgo()),
            },
            _ => Err(SignError::WrongAlgo()),
        }
    }
    /// Check signature of entity
    fn verify(&self) -> Result<(), SigError> {
        if let Some(signature) = self.signature() {
            match self.issuer_pubkey() {
                PubKey::Ed25519(pubkey) => match signature {
                    Sig::Ed25519(sig) => {
                        if pubkey.verify(&self.as_signable_text().as_bytes(), &sig) {
                            Ok(())
                        } else {
                            Err(SigError::InvalidSig())
                        }
                    }
                    _ => Err(SigError::NotSameAlgo()),
                },
                _ => Err(SigError::NotSameAlgo()),
            }
        } else {
            Err(SigError::NotSig())
        }
    }
}

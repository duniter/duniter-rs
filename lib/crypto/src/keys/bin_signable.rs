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

//! Generic code for signing data in binary format

use super::*;
use serde::{Deserialize, Serialize};

/// Signatureable in binary format
pub trait BinSignable<'de>: Serialize + Deserialize<'de> {
    /// Error when serialize self into binary
    type SerdeError: std::error::Error;

    /// Return entity issuer pubkey
    fn issuer_pubkey(&self) -> PubKey;
    /// Return signature
    fn signature(&self) -> Option<Sig>;
    /// Change signature
    fn set_signature(&mut self, _signature: Sig);
    /// Get binary datas without signature
    fn get_bin_without_sig(&self) -> Result<Vec<u8>, Self::SerdeError>;
    /// Add signature to bin datas
    fn add_sig_to_bin_datas(&self, bin_datas: &mut Vec<u8>);
    /// Sign entity with a signator
    fn sign(&mut self, signator: &SignatorEnum) -> Result<Vec<u8>, SignError> {
        if self.signature().is_some() {
            return Err(SignError::AlreadySign);
        }
        match self.issuer_pubkey() {
            PubKey::Ed25519(_) => {
                let mut bin_msg = self
                    .get_bin_without_sig()
                    .map_err(|e| SignError::SerdeError(e.to_string()))?;
                let sig = signator.sign(&bin_msg);
                self.set_signature(sig);
                self.add_sig_to_bin_datas(&mut bin_msg);
                Ok(bin_msg)
            }
            _ => Err(SignError::WrongAlgo),
        }
    }
    /// Check signature of entity
    fn verify(&self) -> Result<(), SigError> {
        if let Some(signature) = self.signature() {
            match self.issuer_pubkey() {
                PubKey::Ed25519(pubkey) => match signature {
                    Sig::Ed25519(sig) => {
                        let signed_part: Vec<u8> = self
                            .get_bin_without_sig()
                            .map_err(|e| SigError::SerdeError(format!("{}", e)))?;
                        pubkey.verify(&signed_part, &sig)
                        /*
                        if pubkey.verify(&signed_part, &sig) {
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
    use bincode;

    #[derive(Deserialize, Serialize)]
    struct BinSignableTestImpl {
        datas: Vec<u8>,
        issuer: PubKey,
        sig: Option<Sig>,
    }

    impl BinSignable<'_> for BinSignableTestImpl {
        type SerdeError = bincode::Error;

        #[inline]
        fn add_sig_to_bin_datas(&self, bin_datas: &mut Vec<u8>) {
            bin_datas
                .extend_from_slice(&bincode::serialize(&self.sig).expect("Fail to binarize sig !"));
        }
        #[inline]
        fn get_bin_without_sig(&self) -> Result<Vec<u8>, bincode::Error> {
            let mut bin_msg = bincode::serialize(&self)?;
            let sig_size = bincode::serialized_size(&self.signature())?;
            let bin_msg_len = bin_msg.len();
            bin_msg.truncate(bin_msg_len - (sig_size as usize));
            Ok(bin_msg)
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
    fn test_bin_signable() {
        let key_pair = ed25519::KeyPairFromSeed32Generator::generate(Seed32::new([
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15,
        ]));

        let signator = SignatorEnum::Ed25519(
            key_pair
                .generate_signator()
                .expect("fail to generate signator !"),
        );

        let mut bin_signable_datas = BinSignableTestImpl {
            datas: vec![0, 1, 2, 3],
            issuer: PubKey::Ed25519(key_pair.pubkey),
            sig: None,
        };

        assert_eq!(Err(SigError::NotSig), bin_signable_datas.verify());

        let _bin_msg = bin_signable_datas
            .sign(&signator)
            .expect("Fail to sign datas !");

        assert_eq!(
            Err(SignError::AlreadySign),
            bin_signable_datas.sign(&signator)
        );

        assert_eq!(Ok(()), bin_signable_datas.verify())
    }
}

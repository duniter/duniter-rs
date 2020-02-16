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

//! Provide wrappers around public keys, private keys and signatures.
//!
//! - Keys can be converted to/from Base58 string format.
//! - Signatures can be converted to/from Base64 string format.
//!
//! # Usage
//!
//! ```
//! use dup_crypto::keys::{KeyPair, PublicKey, Signator, Signature};
//! use dup_crypto::keys::ed25519::{KeyPairFromSaltedPasswordGenerator, SaltedPassword};
//!
//! let generator = KeyPairFromSaltedPasswordGenerator::with_default_parameters();
//!
//! let keypair = generator.generate(SaltedPassword::new(
//!     "salt".to_owned(),
//!     "password".to_owned(),
//! ));
//!
//! let signator = keypair.generate_signator().expect("keypair corrupted");
//!
//! let message = "Hello, world!";
//!
//! let signature = signator.sign(&message.as_bytes());
//!
//! assert!(keypair.pubkey.verify(&message.as_bytes(), &signature).is_ok());
//! ```
//!
//! # Format
//!
//! - Base58 use the following alphabet :
//! `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`
//! - Base64 use the following alphabet :
//! `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/`
//! with `=` as padding character.

pub mod bin_signable;
pub mod ed25519;
pub mod text_signable;

pub use crate::seeds::Seed32;

use crate::bases::b58::ToBase58;
use crate::bases::BaseConvertionError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::str::FromStr;
use thiserror::Error;

/// Cryptographic keys algorithms list
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum KeysAlgo {
    /// Ed25519 algorithm
    Ed25519 = 0,
    /// Schnorr algorithm
    Schnorr = 1,
}

/// Get the cryptographic algorithm.
pub trait GetKeysAlgo: Clone + Debug + PartialEq + Eq {
    /// Get the cryptographic algorithm.
    fn algo(&self) -> KeysAlgo;
}

/// Errors enumeration for signature verification.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum SigError {
    /// Signature and pubkey are not the same algo
    #[error("Signature and pubkey are not the same algo.")]
    NotSameAlgo,
    /// Invalid signature
    #[error("Invalid signature.")]
    InvalidSig,
    /// Absence of signature
    #[error("Absence of signature.")]
    NotSig,
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerdeError(String),
}

/// SignError
#[derive(Debug, Eq, Error, PartialEq)]
pub enum SignError {
    /// Corrupted key pair
    #[error("Corrupted key pair.")]
    CorruptedKeyPair,
    /// WrongAlgo
    #[error("Wrong algo.")]
    WrongAlgo,
    /// WrongPrivkey
    #[error("Wrong private key.")]
    WrongPrivkey,
    /// AlreadySign
    #[error("Already signed.")]
    AlreadySign,
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerdeError(String),
}

/// Define the operations that can be performed on a cryptographic signature.
///
/// A signature can be converted from/to Base64 format.
/// When converted back and forth the value should be the same.
///
/// A signature can be made with a [`PrivateKey`]
/// and a message, and verified with the associated [`PublicKey`].
///
/// [`PrivateKey`]: trait.PrivateKey.html
/// [`PublicKey`]: trait.PublicKey.html
pub trait Signature: Clone + Display + Debug + PartialEq + Eq + Hash {
    /// Create a `Signature` from a Base64 string.
    ///
    /// The Base64 string should contains only valid Base64 characters
    /// and have a correct length (64 bytes when converted). If it's not the case,
    /// a [`BaseConvertionError`] is returned with the corresponding variant.
    ///
    /// [`BaseConvertionError`]: enum.BaseConvertionError.html
    fn from_base64(base64_string: &str) -> Result<Self, BaseConvertionError>;

    /// Convert Signature into butes vector
    fn to_bytes_vector(&self) -> Vec<u8>;

    /// Encode the signature into Base64 string format.
    fn to_base64(&self) -> String;
}

/// Store a cryptographic signature.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Sig {
    /// Store a ed25519 Signature
    Ed25519(ed25519::Signature),
    /// Store a Schnorr Signature
    Schnorr(),
}

impl Sig {
    /// Get Sig size in bytes
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            Sig::Ed25519(_) => *ed25519::SIG_SIZE_IN_BYTES + 2,
            Sig::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

impl Display for Sig {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_base64())
    }
}

impl GetKeysAlgo for Sig {
    fn algo(&self) -> KeysAlgo {
        match *self {
            Sig::Ed25519(_) => KeysAlgo::Ed25519,
            Sig::Schnorr() => KeysAlgo::Schnorr,
        }
    }
}

impl Signature for Sig {
    #[cfg_attr(tarpaulin, skip)]
    fn from_base64(_base64_string: &str) -> Result<Self, BaseConvertionError> {
        unimplemented!()
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        match *self {
            Sig::Ed25519(ed25519_sig) => ed25519_sig.to_bytes_vector(),
            Sig::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn to_base64(&self) -> String {
        match *self {
            Sig::Ed25519(ed25519_sig) => ed25519_sig.to_base64(),
            Sig::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

/// Define the operations that can be performed on a cryptographic public key.
///
/// A `PublicKey` can be converted from/to Base64 format.
/// When converted back and forth the value should be the same.
///
/// A `PublicKey` is used to verify the signature of a message
/// with the associated [`PrivateKey`].
///
/// [`PrivateKey`]: trait.PrivateKey.html
pub trait PublicKey: Clone + Display + Debug + PartialEq + Eq + Hash + ToBase58 {
    /// Signature type of associated cryptosystem.
    type Signature: Signature;

    /// Create a PublicKey from a Base58 string.
    ///
    /// The Base58 string should contains only valid Base58 characters
    /// and have a correct length. If it's not the case,
    /// a [`BaseConvertionError`] is returned with the corresponding variant.
    ///
    /// [`BaseConvertionError`]: enum.BaseConvertionError.html
    fn from_base58(base58_string: &str) -> Result<Self, BaseConvertionError>;

    /// Convert into bytes vector
    fn to_bytes_vector(&self) -> Vec<u8>;

    /// Verify a signature with this public key.
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError>;
}

/// Store a cryptographic public key.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PubKey {
    /// Store a ed25519 public key.
    Ed25519(ed25519::PublicKey),
    /// Store a Schnorr public key.
    Schnorr(),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Error when parsing pubkey bytes
pub enum PubkeyFromBytesError {
    /// Invalid bytes length
    InvalidBytesLen {
        /// Expected length
        expected: usize,
        /// Found length
        found: usize,
    },
}

impl PubKey {
    /// Create pubkey from bytes
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PubkeyFromBytesError> {
        Ok(PubKey::Ed25519(ed25519::PublicKey::try_from(bytes)?))
    }
    /// Compute PubKey size in bytes
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            PubKey::Ed25519(_) => ed25519::PUBKEY_SIZE_IN_BYTES + 3,
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

impl Default for PubKey {
    fn default() -> Self {
        PubKey::Ed25519(ed25519::PublicKey::default())
    }
}

impl GetKeysAlgo for PubKey {
    fn algo(&self) -> KeysAlgo {
        match *self {
            PubKey::Ed25519(_) => KeysAlgo::Ed25519,
            PubKey::Schnorr() => KeysAlgo::Schnorr,
        }
    }
}

impl ToBase58 for PubKey {
    fn to_base58(&self) -> String {
        match *self {
            PubKey::Ed25519(ed25519_pub) => ed25519_pub.to_base58(),
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

impl Display for PubKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl FromStr for PubKey {
    type Err = BaseConvertionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ed25519::PublicKey::from_base58(s).map(PubKey::Ed25519)
    }
}

impl PublicKey for PubKey {
    type Signature = Sig;

    #[cfg_attr(tarpaulin, skip)]
    fn from_base58(_base58_string: &str) -> Result<Self, BaseConvertionError> {
        unimplemented!()
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        match *self {
            PubKey::Ed25519(ed25519_pubkey) => ed25519_pubkey.to_bytes_vector(),
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError> {
        match *self {
            PubKey::Ed25519(ed25519_pubkey) => {
                if let Sig::Ed25519(ed25519_sig) = signature {
                    ed25519_pubkey.verify(message, ed25519_sig)
                } else {
                    Err(SigError::NotSameAlgo)
                }
            }
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

/// Define the operations that can be performed on a cryptographic key pair.
pub trait KeyPair: Clone + Display + Debug + PartialEq + Eq {
    /// Signator type of associated cryptosystem.
    type Signator: Signator;

    /// Generate signator.
    fn generate_signator(&self) -> Result<Self::Signator, SignError>;

    /// Get `PublicKey`
    fn public_key(&self) -> <Self::Signator as Signator>::PublicKey;

    /// Get `Seed32`
    fn seed(&self) -> &Seed32;

    /// Verify a signature with public key.
    fn verify(
        &self,
        message: &[u8],
        signature: &<Self::Signator as Signator>::Signature,
    ) -> Result<(), SigError>;
}

/// Define the operations that can be performed on a cryptographic signator.
pub trait Signator: Debug {
    /// Signature type of associated cryptosystem.
    type Signature: Signature;
    /// PublicKey type of associated cryptosystem.
    type PublicKey: PublicKey;

    /// Get `PublicKey`
    fn public_key(&self) -> Self::PublicKey;

    /// Sign a message with private key encasuled in signator.
    fn sign(&self, message: &[u8]) -> Self::Signature;
}

/// Store a cryptographic key pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyPairEnum {
    /// Store a ed25519 key pair.
    Ed25519(ed25519::Ed25519KeyPair),
    /// Store a Schnorr key pair.
    Schnorr(),
}

impl GetKeysAlgo for KeyPairEnum {
    fn algo(&self) -> KeysAlgo {
        match *self {
            KeyPairEnum::Ed25519(_) => KeysAlgo::Ed25519,
            KeyPairEnum::Schnorr() => KeysAlgo::Schnorr,
        }
    }
}

impl Display for KeyPairEnum {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            KeyPairEnum::Ed25519(ref ed25519_keypair) => {
                write!(f, "({}, hidden)", ed25519_keypair.pubkey.to_base58())
            }
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

impl KeyPair for KeyPairEnum {
    type Signator = SignatorEnum;

    fn generate_signator(&self) -> Result<Self::Signator, SignError> {
        match self {
            KeyPairEnum::Ed25519(ref ed25519_keypair) => {
                Ok(SignatorEnum::Ed25519(ed25519_keypair.generate_signator()?))
            }
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn public_key(&self) -> <Self::Signator as Signator>::PublicKey {
        match self {
            KeyPairEnum::Ed25519(ref ed25519_keypair) => {
                PubKey::Ed25519(ed25519_keypair.public_key())
            }
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn seed(&self) -> &Seed32 {
        match *self {
            KeyPairEnum::Ed25519(ref ed25519_keypair) => &ed25519_keypair.seed(),
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Sig) -> Result<(), SigError> {
        match self {
            KeyPairEnum::Ed25519(ref ed25519_keypair) => {
                if let Sig::Ed25519(ed25519_sig) = signature {
                    ed25519_keypair.verify(message, ed25519_sig)
                } else {
                    Err(SigError::NotSameAlgo)
                }
            }
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

/// Store a cryptographic signator.
#[derive(Debug)]
pub enum SignatorEnum {
    /// Store a ed25519 signator.
    Ed25519(ed25519::Signator),
    /// Store a Schnorr signator.
    Schnorr(),
}

impl Signator for SignatorEnum {
    type PublicKey = PubKey;
    type Signature = Sig;

    fn public_key(&self) -> Self::PublicKey {
        match self {
            SignatorEnum::Ed25519(ref ed25519_signator) => {
                PubKey::Ed25519(ed25519_signator.public_key())
            }
            SignatorEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }

    fn sign(&self, message: &[u8]) -> Sig {
        match self {
            SignatorEnum::Ed25519(ref ed25519_signator) => {
                Sig::Ed25519(ed25519_signator.sign(message))
            }
            SignatorEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use unwrap::unwrap;

    pub fn valid_key_pair_1() -> KeyPairEnum {
        KeyPairEnum::Ed25519(ed25519::KeyPairFromSeed32Generator::generate(Seed32::new(
            [
                59u8, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101,
                50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41,
            ],
        )))
    }

    #[test]
    fn sig() {
        let sig_bytes = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let sig_str_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==".to_owned();
        let sig = Sig::Ed25519(ed25519::Signature(sig_bytes));

        assert_eq!(sig.size_in_bytes(), *ed25519::SIG_SIZE_IN_BYTES + 2);
        assert_eq!(sig_str_b64, format!("{}", sig));

        assert_eq!(KeysAlgo::Ed25519, sig.algo());
        assert_eq!(KeysAlgo::Schnorr, Sig::Schnorr().algo());

        assert_eq!(sig_bytes.to_vec(), sig.to_bytes_vector());

        assert_eq!(sig_str_b64, sig.to_base64());
    }

    #[test]
    fn pubkey() {
        let pubkey_default = PubKey::default();
        let pubkey_bytes = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let pubkey = PubKey::Ed25519(unwrap!(ed25519::PublicKey::try_from(&pubkey_bytes[..])));
        assert_eq!(pubkey_default, pubkey);

        let pubkey_str_b58 = "11111111111111111111111111111111".to_owned();
        assert_eq!(
            pubkey_default,
            PubKey::from_str(&pubkey_str_b58).expect("Fail to parse pubkey !")
        );

        assert_eq!(pubkey.size_in_bytes(), *ed25519::PUBKEY_SIZE_IN_BYTES + 3);
        assert_eq!(pubkey_str_b58, format!("{}", pubkey));

        assert_eq!(KeysAlgo::Ed25519, pubkey.algo());
        assert_eq!(KeysAlgo::Schnorr, PubKey::Schnorr().algo());

        assert_eq!(pubkey_bytes.to_vec(), pubkey.to_bytes_vector());

        assert_eq!(pubkey_str_b58, pubkey.to_base58());

        assert_eq!(
            Err(SigError::InvalidSig),
            pubkey.verify(
                b"message",
                &Sig::Ed25519(ed25519::Signature([
                    0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ]))
            )
        )
    }

    #[test]
    fn seed() {
        let seed_default = Seed32::default();
        let seed_bytes = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let seed_str_b58 = "11111111111111111111111111111111".to_owned();
        let seed = Seed32::new(seed_bytes);

        assert_eq!(seed_default, seed);
        assert_eq!(
            seed_default,
            Seed32::from_base58(&seed_str_b58).expect("Fail to parse seed !")
        );

        assert_eq!(seed_str_b58, format!("{}", seed));

        assert_eq!(seed_str_b58, seed.to_base58());
    }

    fn false_key_pair_ed25519() -> ed25519::Ed25519KeyPair {
        ed25519::KeyPairFromSeed32Generator::generate(Seed32::new([
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]))
    }

    #[test]
    fn key_pair() {
        let false_key_pair_ed25519 = false_key_pair_ed25519();
        let false_key_pair = KeyPairEnum::Ed25519(false_key_pair_ed25519.clone());

        assert_eq!(KeysAlgo::Ed25519, false_key_pair.algo());
        assert_eq!(KeysAlgo::Schnorr, KeyPairEnum::Schnorr().algo());
        assert_eq!(
            "(4zvwRjXUKGfvwnParsHAS3HuSVzV5cA4McphgmoCtajS, hidden)".to_owned(),
            format!("{}", false_key_pair)
        );
        assert_eq!(
            PubKey::Ed25519(false_key_pair_ed25519.pubkey),
            false_key_pair.public_key()
        );
        assert_eq!(false_key_pair_ed25519.seed, false_key_pair.seed().clone());
        assert_eq!(
            Err(SigError::InvalidSig),
            false_key_pair.verify(
                b"message",
                &Sig::Ed25519(ed25519::Signature([
                    0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]))
            )
        );
    }

    #[test]
    fn key_pair_verify_wrong_sig_algo() {
        let false_key_pair_ed25519 = false_key_pair_ed25519();
        let false_key_pair = KeyPairEnum::Ed25519(false_key_pair_ed25519);
        assert_eq!(
            Err(SigError::NotSameAlgo),
            false_key_pair.verify(b"message", &Sig::Schnorr()),
        );
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn sig_schnorr_size() {
        let sig = Sig::Schnorr();
        sig.size_in_bytes();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn sig_schnorr_to_bytes() {
        let sig = Sig::Schnorr();
        sig.to_bytes_vector();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn sig_schnorr_to_base64() {
        let sig = Sig::Schnorr();
        sig.to_base64();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn pubkey_schnorr_size() {
        let pubkey = PubKey::Schnorr();
        pubkey.size_in_bytes();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn pubkey_schnorr_base58() {
        let pubkey = PubKey::Schnorr();
        pubkey.to_base58();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn pubkey_schnorr_to_bytes() {
        let pubkey = PubKey::Schnorr();
        pubkey.to_bytes_vector();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn pubkey_schnorr_verify() {
        let pubkey = PubKey::Schnorr();
        let _ = pubkey.verify(b"message", &Sig::Schnorr());
    }

    #[test]
    fn pubkey_verify_sig_wrong_algo() {
        let pubkey = PubKey::default();
        assert_eq!(
            Err(SigError::NotSameAlgo),
            pubkey.verify(b"message", &Sig::Schnorr()),
        );
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_display() {
        let key_pair = KeyPairEnum::Schnorr();
        format!("{}", key_pair);
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_generate_signator() {
        let _ = KeyPairEnum::Schnorr().generate_signator();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_get_pubkey() {
        let key_pair = KeyPairEnum::Schnorr();
        key_pair.public_key();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_get_seed() {
        let key_pair = KeyPairEnum::Schnorr();
        key_pair.seed();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_verify() {
        let key_pair = KeyPairEnum::Schnorr();
        let _ = key_pair.verify(b"message", &Sig::Schnorr());
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn signator_schnorr_get_pubkey() {
        let signator = SignatorEnum::Schnorr();
        signator.public_key();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn signator_schnorr_sign() {
        let signator = SignatorEnum::Schnorr();
        signator.sign(b"message");
    }

    #[test]
    fn pubkey_from_bytes() {
        assert_eq!(
            Err(PubkeyFromBytesError::InvalidBytesLen {
                expected: *ed25519::PUBKEY_SIZE_IN_BYTES,
                found: 2,
            }),
            PubKey::from_bytes(&[0u8, 1]),
        );
        assert_eq!(
            Err(PubkeyFromBytesError::InvalidBytesLen {
                expected: *ed25519::PUBKEY_SIZE_IN_BYTES,
                found: 33,
            }),
            PubKey::from_bytes(&[
                0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 31
            ]),
        );
        assert_eq!(
            Ok(PubKey::Ed25519(ed25519::PublicKey::default())),
            PubKey::from_bytes(&[0u8; 32]),
        );
    }
}

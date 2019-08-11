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
//! use dup_crypto::keys::{Signature, PublicKey, PrivateKey, KeyPair};
//! use dup_crypto::keys::ed25519::KeyPairFromSaltedPasswordGenerator;
//!
//! let generator = KeyPairFromSaltedPasswordGenerator::with_default_parameters();
//!
//! let keypair = generator.generate(
//!     b"password",
//!     b"salt"
//! );
//!
//! let message = "Hello, world!";
//!
//! let signature = keypair.sign(&message.as_bytes());
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

use crate::bases::BaseConvertionError;
use base58::ToBase58;
use bincode;
use durs_common_tools::fatal_error;
use failure::Fail;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::str::FromStr;

pub mod bin_signable;
pub mod ed25519;
pub mod text_signable;

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
#[derive(Debug, Eq, Fail, PartialEq)]
pub enum SigError {
    /// Signature and pubkey are not the same algo
    #[fail(display = "Signature and pubkey are not the same algo.")]
    NotSameAlgo,
    /// Invalid signature
    #[fail(display = "Invalid signature.")]
    InvalidSig,
    /// Absence of signature
    #[fail(display = "Absence of signature.")]
    NotSig,
    /// Serialization error
    #[fail(display = "Serialization error: {}", _0)]
    SerdeError(String),
}

/// SignError
#[derive(Debug, Eq, Fail, PartialEq)]
pub enum SignError {
    /// WrongAlgo
    #[fail(display = "Wrong algo.")]
    WrongAlgo,
    /// WrongPrivkey
    #[fail(display = "Wrong private key.")]
    WrongPrivkey,
    /// AlreadySign
    #[fail(display = "Already signed.")]
    AlreadySign,
    /// Serialization error
    #[fail(display = "Serialization error: {}", _0)]
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
            Sig::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
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
            Sig::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
    fn to_base64(&self) -> String {
        match *self {
            Sig::Ed25519(ed25519_sig) => ed25519_sig.to_base64(),
            Sig::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
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

impl PubKey {
    /// Compute PubKey size in bytes
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            PubKey::Ed25519(_) => ed25519::PUBKEY_SIZE_IN_BYTES + 3,
            PubKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

impl Default for PubKey {
    fn default() -> Self {
        PubKey::Ed25519(ed25519::PublicKey([
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]))
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
            PubKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
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
            PubKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError> {
        match *self {
            PubKey::Ed25519(ed25519_pubkey) => {
                if let Sig::Ed25519(ed25519_sig) = signature {
                    ed25519_pubkey.verify(message, ed25519_sig)
                } else {
                    fatal_error!("Try to verify a signature with public key of a different algorithm !\nSignature={:?}\nPublickey={:?}", signature, self)
                }
            }
            PubKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

/// Define the operations that can be performed on a cryptographic private key.
///
/// A `PrivateKey` can be converted from/to Base58 format.
/// When converted back and forth the value should be the same.
///
/// A `PrivateKey` is used to sign a message. The corresponding [`PublicKey`]
/// will then be used to verify that signature.
///
/// [`PublicKey`]: trait.PublicKey.html
pub trait PrivateKey: Clone + Display + Debug + PartialEq + Eq + ToBase58 {
    /// Signature type of associated cryptosystem.
    type Signature: Signature;

    /// Create a PrivateKey from a Base58 string.
    ///
    /// The Base58 string should contains only valid Base58 characters
    /// and have a correct length. If it's not the case, a [`BaseConvertionError`]
    /// is returned with the corresponding variant.
    ///
    /// [`BaseConvertionError`]: enum.BaseConvertionError.html
    fn from_base58(base58_string: &str) -> Result<Self, BaseConvertionError>;

    /// Sign a message with this private key.
    fn sign(&self, message: &[u8]) -> Self::Signature;
}

/// Store a cryptographic private key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivKey {
    /// Store a ed25519 private key.
    Ed25519(ed25519::PrivateKey),
    /// Store a Schnorr private key.
    Schnorr(),
}

impl GetKeysAlgo for PrivKey {
    fn algo(&self) -> KeysAlgo {
        match *self {
            PrivKey::Ed25519(_) => KeysAlgo::Ed25519,
            PrivKey::Schnorr() => KeysAlgo::Schnorr,
        }
    }
}

impl ToBase58 for PrivKey {
    fn to_base58(&self) -> String {
        match *self {
            PrivKey::Ed25519(ed25519_privkey) => ed25519_privkey.to_base58(),
            PrivKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

impl Display for PrivKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl PrivateKey for PrivKey {
    type Signature = Sig;

    #[cfg_attr(tarpaulin, skip)]
    fn from_base58(_base58_string: &str) -> Result<Self, BaseConvertionError> {
        unimplemented!()
    }
    fn sign(&self, message: &[u8]) -> Self::Signature {
        match *self {
            PrivKey::Ed25519(ed25519_privkey) => Sig::Ed25519(ed25519_privkey.sign(message)),
            PrivKey::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

/// Define the operations that can be performed on a cryptographic key pair.
pub trait KeyPair: Clone + Display + Debug + PartialEq + Eq {
    /// Signature type of associated cryptosystem.
    type Signature: Signature;
    /// PublicKey type of associated cryptosystem.
    type PublicKey: PublicKey;
    /// PrivateKey type of associated cryptosystem.
    type PrivateKey: PrivateKey;

    /// Get `PublicKey`
    fn public_key(&self) -> Self::PublicKey;

    /// Get `PrivateKey`
    fn private_key(&self) -> Self::PrivateKey;

    /// Sign a message with private key.
    fn sign(&self, message: &[u8]) -> Self::Signature;

    /// Verify a signature with public key.
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError>;
}

/// Store a cryptographic key pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyPairEnum {
    /// Store a ed25519 key pair.
    Ed25519(ed25519::KeyPair),
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
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => {
                write!(f, "({}, hidden)", ed25519_keypair.pubkey.to_base58())
            }
            KeyPairEnum::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

impl KeyPair for KeyPairEnum {
    type Signature = Sig;
    type PublicKey = PubKey;
    type PrivateKey = PrivKey;

    fn public_key(&self) -> Self::PublicKey {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => PubKey::Ed25519(ed25519_keypair.public_key()),
            KeyPairEnum::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
    fn private_key(&self) -> Self::PrivateKey {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => {
                PrivKey::Ed25519(ed25519_keypair.private_key())
            }
            KeyPairEnum::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Sig) -> Result<(), SigError> {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => {
                if let Sig::Ed25519(ed25519_sig) = signature {
                    ed25519_keypair.verify(message, ed25519_sig)
                } else {
                    fatal_error!("Try to verify a signature with key pair of a different algorithm !\nSignature={:?}\nKeyPair={}", signature, self)
                }
            }
            KeyPairEnum::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
    fn sign(&self, message: &[u8]) -> Sig {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => Sig::Ed25519(ed25519_keypair.sign(message)),
            KeyPairEnum::Schnorr() => fatal_error!("Schnorr algo not yet supported !"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    pub fn valid_key_pair_1() -> KeyPairEnum {
        let kp = KeyPairEnum::Ed25519(ed25519::KeyPair {
            pubkey: ed25519::PublicKey([
                59u8, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13, 115, 101,
                50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41,
            ]),
            privkey: ed25519::PrivateKey([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 59, 106, 39, 188, 206, 182, 164, 45, 98, 163, 168, 208, 42, 111, 13,
                115, 101, 50, 21, 119, 29, 226, 67, 166, 58, 192, 72, 161, 139, 89, 218, 41,
            ]),
        });
        println!("kp.pub={:?}", kp.public_key().to_bytes_vector());
        if let PrivKey::Ed25519(ed_pk) = kp.private_key() {
            println!("kp.priv={:?}", ed_pk.0.to_vec());
        }
        kp
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
        let pubkey_str_b58 = "11111111111111111111111111111111".to_owned();
        let pubkey = PubKey::Ed25519(ed25519::PublicKey(pubkey_bytes));

        assert_eq!(pubkey_default, pubkey);
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
    fn privkey() {
        let privkey_bytes = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        let privkey_str_b58 =
            "1111111111111111111111111111111111111111111111111111111111111111".to_owned();
        let privkey = PrivKey::Ed25519(ed25519::PrivateKey(privkey_bytes));

        assert_eq!(privkey_str_b58, format!("{}", privkey));

        assert_eq!(KeysAlgo::Ed25519, privkey.algo());
        assert_eq!(KeysAlgo::Schnorr, PrivKey::Schnorr().algo());

        assert_eq!(privkey_str_b58, privkey.to_base58());

        assert_eq!(
            Sig::Ed25519(ed25519::Signature::from_base64("JPurBgnHExHND1woow9nB7xVQjKkdHGs1znQbgv0ttboJXueHKd4SOvxuNWmw4w07F4CT//olYMEBw51Cy0SDA==").unwrap()),
            privkey.sign(b"message")
        );
    }

    fn false_key_pair_ed25519() -> ed25519::KeyPair {
        ed25519::KeyPair {
            pubkey: ed25519::PublicKey([
                0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ]),
            privkey: ed25519::PrivateKey([
                0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]),
        }
    }

    #[test]
    fn key_pair() {
        let false_key_pair_ed25519 = false_key_pair_ed25519();
        let false_key_pair = KeyPairEnum::Ed25519(false_key_pair_ed25519);

        assert_eq!(KeysAlgo::Ed25519, false_key_pair.algo());
        assert_eq!(KeysAlgo::Schnorr, KeyPairEnum::Schnorr().algo());
        assert_eq!(
            "(11111111111111111111111111111111, hidden)".to_owned(),
            format!("{}", false_key_pair)
        );
        assert_eq!(
            PubKey::Ed25519(false_key_pair_ed25519.pubkey),
            false_key_pair.public_key()
        );
        assert_eq!(
            PrivKey::Ed25519(false_key_pair_ed25519.privkey),
            false_key_pair.private_key()
        );
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
    #[should_panic(
        expected = "Try to verify a signature with key pair of a different algorithm !\n\
                    Signature=Schnorr\nKeyPair=(11111111111111111111111111111111, hidden)"
    )]
    fn key_pair_verify_wrong_sig_algo() {
        let false_key_pair_ed25519 = false_key_pair_ed25519();
        let false_key_pair = KeyPairEnum::Ed25519(false_key_pair_ed25519);
        let _ = false_key_pair.verify(b"message", &Sig::Schnorr());
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
    #[should_panic(
        expected = "Try to verify a signature with public key of a different algorithm !\n\
        Signature=Schnorr\nPublickey=Ed25519(PublicKey { 11111111111111111111111111111111 })"
    )]
    fn pubkey_verify_sig_wrong_algo() {
        let pubkey = PubKey::default();
        let _ = pubkey.verify(b"message", &Sig::Schnorr());
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn privkey_schnorr_base58() {
        let privkey = PrivKey::Schnorr();
        privkey.to_base58();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn privkey_schnorr_sign() {
        let privkey = PrivKey::Schnorr();
        privkey.sign(b"message");
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_display() {
        let key_pair = KeyPairEnum::Schnorr();
        format!("{}", key_pair);
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_get_pubkey() {
        let key_pair = KeyPairEnum::Schnorr();
        key_pair.public_key();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_get_privkey() {
        let key_pair = KeyPairEnum::Schnorr();
        key_pair.private_key();
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_verify() {
        let key_pair = KeyPairEnum::Schnorr();
        let _ = key_pair.verify(b"message", &Sig::Schnorr());
    }

    #[test]
    #[should_panic(expected = "Schnorr algo not yet supported !")]
    fn key_pair_schnorr_sign() {
        let key_pair = KeyPairEnum::Schnorr();
        key_pair.sign(b"message");
    }
}

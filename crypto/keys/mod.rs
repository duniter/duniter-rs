//  Copyright (C) 2018  The Duniter Project Developers.
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
//! use duniter_crypto::keys::{Signature, PublicKey, PrivateKey, KeyPair};
//! use duniter_crypto::keys::ed25519::KeyPairFromSaltedPasswordGenerator;
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
//! assert!(keypair.pubkey.verify(&message.as_bytes(), &signature));
//! ```
//!
//! # Format
//!
//! - Base58 use the following alphabet :
//! `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`
//! - Base64 use the following alphabet :
//! `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/`
//! with `=` as padding character.

extern crate serde;

use base58::ToBase58;
use bincode;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;

pub mod bin_signable;
pub mod ed25519;

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

/// Errors enumeration for Base58/64 strings convertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseConvertionError {
    /// Data have invalid key length (found, expected).
    InvalidKeyLendth(usize, usize),
    /// Base58 have an invalid character.
    InvalidCharacter(char, usize),
    /// Base58 have invalid lendth
    InvalidBaseConverterLength(),
}

/// Errors enumeration for signature verification.
#[derive(Debug)]
pub enum SigError {
    /// Signature and pubkey are not the same algo
    NotSameAlgo(),
    /// Invalid signature
    InvalidSig(),
    /// Absence of signature
    NotSig(),
    /// Deserialization error
    DeserError(bincode::Error),
}

impl From<bincode::Error> for SigError {
    fn from(e: bincode::Error) -> Self {
        SigError::DeserError(e)
    }
}

/// SignError
#[derive(Debug, Copy, Clone)]
pub enum SignError {
    /// WrongAlgo
    WrongAlgo(),
    /// WrongPrivkey
    WrongPrivkey(),
    /// AlreadySign
    AlreadySign(),
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
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
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
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

impl Default for PubKey {
    fn default() -> Self {
        PubKey::Schnorr()
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

impl PublicKey for PubKey {
    type Signature = Sig;

    fn from_base58(_base58_string: &str) -> Result<Self, BaseConvertionError> {
        unimplemented!()
    }
    fn to_bytes_vector(&self) -> Vec<u8> {
        match *self {
            PubKey::Ed25519(ed25519_pubkey) => ed25519_pubkey.to_bytes_vector(),
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool {
        match *self {
            PubKey::Ed25519(ed25519_pubkey) => if let Sig::Ed25519(ed25519_sig) = signature {
                ed25519_pubkey.verify(message, ed25519_sig)
            } else {
                panic!("Try to verify a signature with public key of a different algorithm !\nSignature={:?}\nPublickey={:?}", signature, self)
            },
            PubKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
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
            PrivKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
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
    fn from_base58(_base58_string: &str) -> Result<Self, BaseConvertionError> {
        unimplemented!()
    }
    fn sign(&self, message: &[u8]) -> Self::Signature {
        match *self {
            PrivKey::Ed25519(ed25519_privkey) => Sig::Ed25519(ed25519_privkey.sign(message)),
            PrivKey::Schnorr() => panic!("Schnorr algo not yet supported !"),
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
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
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
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
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
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn private_key(&self) -> Self::PrivateKey {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => {
                PrivKey::Ed25519(ed25519_keypair.private_key())
            }
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn verify(&self, message: &[u8], signature: &Sig) -> bool {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => if let Sig::Ed25519(ed25519_sig) = signature {
                ed25519_keypair.verify(message, ed25519_sig)
            } else {
                panic!("Try to verify a signature with key pair of a different algorithm !\nSignature={:?}\nKeyPair={:?}", signature, self)
            },
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
    fn sign(&self, message: &[u8]) -> Sig {
        match *self {
            KeyPairEnum::Ed25519(ed25519_keypair) => Sig::Ed25519(ed25519_keypair.sign(message)),
            KeyPairEnum::Schnorr() => panic!("Schnorr algo not yet supported !"),
        }
    }
}

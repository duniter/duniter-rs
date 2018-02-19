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
//! use duniter_keys::{Signature, PublicKey, PrivateKey, KeyPair};
//! use duniter_keys::ed25519::KeyPairFromSaltedPasswordGenerator;
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

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
        trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
        unused_qualifications)]

extern crate base58;
extern crate base64;
extern crate crypto;

use std::fmt::Display;
use std::fmt::Debug;

use base58::ToBase58;

pub mod ed25519;

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

/// Store a cryptographic signature.
///
/// A signature can be converted from/to Base64 format.
/// When converted back and forth the value should be the same.
///
/// A signature can be made with a [`PrivateKey`]
/// and a message, and verified with the associated [`PublicKey`].
///
/// [`PrivateKey`]: trait.PrivateKey.html
/// [`PublicKey`]: trait.PublicKey.html
pub trait Signature: Clone + Display + Debug + PartialEq + Eq {
    /// Create a `Signature` from a Base64 string.
    ///
    /// The Base64 string should contains only valid Base64 characters
    /// and have a correct length (64 bytes when converted). If it's not the case,
    /// a [`BaseConvertionError`] is returned with the corresponding variant.
    ///
    /// [`BaseConvertionError`]: enum.BaseConvertionError.html
    fn from_base64(base64_string: &str) -> Result<Self, BaseConvertionError>;

    /// Encode the signature into Base64 string format.
    fn to_base64(&self) -> String;
}

/// Store a cryptographic public key.
///
/// A `PublicKey` can be converted from/to Base64 format.
/// When converted back and forth the value should be the same.
///
/// A `PublicKey` is used to verify the signature of a message
/// with the associated [`PrivateKey`].
///
/// [`PrivateKey`]: trait.PrivateKey.html
pub trait PublicKey: Clone + Display + Debug + PartialEq + Eq + ToBase58 {
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

    /// Verify a signature with this public key.
    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool;
}

/// Store a cryptographic private key.
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

/// Store a cryptographic key pair (`PublicKey` + `PrivateKey`)
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

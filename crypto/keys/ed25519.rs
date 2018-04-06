//  Copyright (C) 2017  The Duniter Project Developers.
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

//! Provide wrappers around ed25519 keys and signatures
//!
//! Key pairs can be generated with [`KeyPairGenerator`].
//!
//! [`KeyPairGenerator`]: struct.KeyPairGenerator.html

extern crate serde;

use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use base58::{FromBase58, FromBase58Error, ToBase58};
use base64;
use base64::DecodeError;
use crypto;
use self::serde::ser::{Serialize, Serializer};

use super::{BaseConvertionError, PrivateKey as PrivateKeyMethods, PublicKey as PublicKeyMethods};

/// Store a ed25519 signature.
#[derive(Clone, Copy)]
pub struct Signature(pub [u8; 64]);

impl Hash for Signature {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        let mut hasher = DefaultHasher::new();
        Hash::hash_slice(&self.0, &mut hasher);
    }
}

impl super::Signature for Signature {
    fn from_base64(base64_data: &str) -> Result<Signature, BaseConvertionError> {
        match base64::decode(base64_data) {
            Ok(result) => {
                if result.len() == 64 {
                    let mut u8_array = [0; 64];

                    u8_array[..64].clone_from_slice(&result[..64]);

                    Ok(Signature(u8_array))
                } else {
                    Err(BaseConvertionError::InvalidKeyLendth(result.len(), 64))
                }
            }
            Err(DecodeError::InvalidByte(pos, byte)) => {
                Err(BaseConvertionError::InvalidCharacter(byte as char, pos))
            }
            Err(DecodeError::InvalidLength) => {
                Err(BaseConvertionError::InvalidBaseConverterLength())
            }
        }
    }

    fn to_base64(&self) -> String {
        base64::encode(&self.0[..]) // need to take a slice for required trait `AsRef<[u8]>`
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use super::Signature;

        write!(f, "{}", self.to_base64())
    }
}

impl Debug for Signature {
    // Signature { 1eubHHb... }
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "Signature {{ {} }}", self)
    }
}

impl PartialEq<Signature> for Signature {
    fn eq(&self, other: &Signature) -> bool {
        // No PartialEq for [u8;64], need to use 2 [u8;32]
        self.0[0..32] == other.0[0..32] && self.0[32..64] == other.0[32..64]
    }
}

impl Eq for Signature {}

/// Store a Ed25519 public key.
///
/// Can be generated with [`KeyPairGenerator`].
///
/// [`KeyPairGenerator`]: struct.KeyPairGenerator.html
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub [u8; 32]);

impl ToBase58 for PublicKey {
    fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for PublicKey {
    // PublicKey { DNann1L... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "PublicKey {{ {} }}", self)
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl super::PublicKey for PublicKey {
    type Signature = Signature;

    fn from_base58(base58_data: &str) -> Result<Self, BaseConvertionError> {
        match base58_data.from_base58() {
            Ok(result) => {
                if result.len() == 32 {
                    let mut u8_array = [0; 32];

                    u8_array[..32].clone_from_slice(&result[..32]);

                    Ok(PublicKey(u8_array))
                } else {
                    Err(BaseConvertionError::InvalidKeyLendth(result.len(), 32))
                }
            }
            Err(FromBase58Error::InvalidBase58Character(character, pos)) => {
                Err(BaseConvertionError::InvalidCharacter(character, pos))
            }
            Err(FromBase58Error::InvalidBase58Length) => {
                Err(BaseConvertionError::InvalidBaseConverterLength())
            }
        }
    }

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool {
        crypto::ed25519::verify(message, &self.0, &signature.0)
    }
}

/// Store a Ed25519 private key.
///
/// Can be generated with [`KeyPairGenerator`].
///
/// [`KeyPairGenerator`]: struct.KeyPairGenerator.html
#[derive(Copy, Clone)]
pub struct PrivateKey(pub [u8; 64]);

impl ToBase58 for PrivateKey {
    fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for PrivateKey {
    // PrivateKey { 468Q1XtT... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "PrivateKey {{ {} }}", self)
    }
}

impl PartialEq<PrivateKey> for PrivateKey {
    fn eq(&self, other: &PrivateKey) -> bool {
        // No PartialEq for [u8;64], need to use 2 [u8;32]
        self.0[0..32] == other.0[0..32] && self.0[32..64] == other.0[32..64]
    }
}

impl Eq for PrivateKey {}

impl super::PrivateKey for PrivateKey {
    type Signature = Signature;

    fn from_base58(base58_data: &str) -> Result<Self, BaseConvertionError> {
        match base58_data.from_base58() {
            Ok(result) => {
                if result.len() == 64 {
                    let mut u8_array = [0; 64];

                    u8_array[..64].clone_from_slice(&result[..64]);

                    Ok(PrivateKey(u8_array))
                } else {
                    Err(BaseConvertionError::InvalidKeyLendth(result.len(), 64))
                }
            }
            Err(FromBase58Error::InvalidBase58Character(character, pos)) => {
                Err(BaseConvertionError::InvalidCharacter(character, pos))
            }
            Err(FromBase58Error::InvalidBase58Length) => {
                Err(BaseConvertionError::InvalidBaseConverterLength())
            }
        }
    }

    /// Sign a message with this private key.
    fn sign(&self, message: &[u8]) -> Self::Signature {
        Signature(crypto::ed25519::signature(message, &self.0))
    }
}

/// Store a ed25519 cryptographic key pair (`PublicKey` + `PrivateKey`)
#[derive(Debug, Copy, Clone)]
pub struct KeyPair {
    /// Store a Ed25519 public key.
    pub pubkey: PublicKey,
    /// Store a Ed25519 private key.
    pub privkey: PrivateKey,
}

impl Display for KeyPair {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, hidden)", self.pubkey.to_base58())
    }
}

impl PartialEq<KeyPair> for KeyPair {
    fn eq(&self, other: &KeyPair) -> bool {
        self.pubkey.eq(&other.pubkey) && self.privkey.eq(&other.privkey)
    }
}

impl Eq for KeyPair {}

impl super::KeyPair for KeyPair {
    type Signature = Signature;
    type PublicKey = PublicKey;
    type PrivateKey = PrivateKey;

    fn public_key(&self) -> PublicKey {
        self.pubkey
    }

    fn private_key(&self) -> PrivateKey {
        self.privkey
    }

    fn sign(&self, message: &[u8]) -> Signature {
        self.private_key().sign(message)
    }

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool {
        self.public_key().verify(message, signature)
    }
}

/// Keypair generator with given parameters for `scrypt` keypair function.
#[derive(Debug, Copy, Clone)]
pub struct KeyPairFromSaltedPasswordGenerator {
    /// The log2 of the Scrypt parameter `N`.
    log_n: u8,
    /// The Scrypt parameter `r`
    r: u32,
    /// The Scrypt parameter `p`
    p: u32,
}

impl KeyPairFromSaltedPasswordGenerator {
    /// Create a `KeyPairGenerator` with default arguments `(log_n: 12, r: 16, p: 1)`
    pub fn with_default_parameters() -> KeyPairFromSaltedPasswordGenerator {
        KeyPairFromSaltedPasswordGenerator {
            log_n: 12,
            r: 16,
            p: 1,
        }
    }

    /// Create a `KeyPairFromSaltedPasswordGenerator` with given arguments.
    ///
    /// # Arguments
    ///
    /// - log_n - The log2 of the Scrypt parameter N
    /// - r - The Scrypt parameter r
    /// - p - The Scrypt parameter p
    pub fn with_parameters(log_n: u8, r: u32, p: u32) -> KeyPairFromSaltedPasswordGenerator {
        KeyPairFromSaltedPasswordGenerator { log_n, r, p }
    }

    /// Create a keypair based on a given password and salt.
    ///
    /// The [`PublicKey`](struct.PublicKey.html) will be able to verify messaged signed with
    /// the [`PrivateKey`](struct.PrivateKey.html).
    pub fn generate(&self, password: &[u8], salt: &[u8]) -> KeyPair {
        let mut seed = [0u8; 32];
        crypto::scrypt::scrypt(
            salt,
            password,
            &crypto::scrypt::ScryptParams::new(self.log_n, self.r, self.p),
            &mut seed,
        );

        let (private, public) = crypto::ed25519::keypair(&seed);
        KeyPair {
            pubkey: PublicKey(public),
            privkey: PrivateKey(private),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keys::{KeyPair, Signature};

    #[test]
    fn base58_private_key() {
        let private58 =
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9r\
             qnXuW3iAfZACm7";
        let private_key = super::PrivateKey::from_base58(private58).unwrap();
        let private_raw = private58.from_base58().unwrap();

        for (key, raw) in private_key.0.iter().zip(private_raw.iter()) {
            assert_eq!(key, raw);
        }

        assert_eq!(private_key.to_base58(), private58);
        assert_eq!(
            private_key,
            super::PrivateKey::from_base58(private58).unwrap()
        );

        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iA\
                fZACm7djh",
            ).unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(67, 64)
        );
        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9",
            ).unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(53, 64)
        );
        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9<<",
            ).unwrap_err(),
            BaseConvertionError::InvalidCharacter('<', 73)
        );
    }

    #[test]
    fn base58_public_key() {
        let public58 = "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV";
        let public_key = super::PublicKey::from_base58(public58).unwrap();
        let public_raw = public58.from_base58().unwrap();

        for (key, raw) in public_key.0.iter().zip(public_raw.iter()) {
            assert_eq!(key, raw);
        }

        assert_eq!(public_key.to_base58(), public58);
        assert_eq!(public_key, super::PublicKey::from_base58(public58).unwrap());

        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLVdjq")
                .unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(35, 32)
        );
        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd")
                .unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(31, 32)
        );
        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd<<")
                .unwrap_err(),
            BaseConvertionError::InvalidCharacter('<', 42)
        );
    }

    #[test]
    fn base64_signature() {
        let signature64 = "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
                           MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==";
        let signature = super::Signature::from_base64(signature64).unwrap();
        let signature_raw = base64::decode(signature64).unwrap();

        for (sig, raw) in signature.0.iter().zip(signature_raw.iter()) {
            assert_eq!(sig, raw);
        }

        assert_eq!(
            super::Signature::from_base64("YmhlaW9iaHNlcGlvaGVvaXNlcGl2ZXBvdm5pc2U=").unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(29, 64)
        );
        assert_eq!(
            super::Signature::from_base64(
                "YmhlaW9iaHNlcGlvaGVvaXNlcGl2ZXBvdm5pc2V2c2JlaW9idmVpb3Zqc\
                 2V2Z3BpaHNlamVwZ25qZXNqb2dwZWpnaW9zZXNkdnNic3JicmJyZGJyZGI=",
            ).unwrap_err(),
            BaseConvertionError::InvalidKeyLendth(86, 64)
        );
        assert_eq!(
            super::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
                 mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAgdha<<",
            ).unwrap_err(),
            BaseConvertionError::InvalidCharacter('<', 89)
        );
    }

    #[test]
    fn message_sign_verify() {
        let pubkey =
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV").unwrap();

        let prikey = super::PrivateKey::from_base58(
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt\
             5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
        ).unwrap();

        let expected_signature = super::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
             MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        ).unwrap();

        let message = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let sig = prikey.sign(message.as_bytes());

        assert_eq!(sig, expected_signature);
        assert!(pubkey.verify(message.as_bytes(), &sig));
    }

    #[test]
    fn keypair_generate_() {
        let keypair = KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        );

        assert_eq!(
            keypair.pubkey.to_string(),
            "7iMV3b6j2hSj5WtrfchfvxivS9swN3opDgxudeHq64fb"
        );
    }

    #[test]
    fn keypair_generate_sign_and_verify() {
        let keypair = KeyPairFromSaltedPasswordGenerator::with_default_parameters()
            .generate("password".as_bytes(), "salt".as_bytes());

        let message = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let sig = keypair.sign(message.as_bytes());
        assert!(keypair.verify(message.as_bytes(), &sig));
    }
}

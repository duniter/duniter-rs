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

//! Provide wrappers around ed25519 keys and signatures
//!
//! Key pairs can be generated with [`KeyPairGenerator`].
//!
//! [`KeyPairGenerator`]: struct.KeyPairGenerator.html

use super::PublicKey as PublicKeyMethods;
use super::{PubkeyFromBytesError, SigError};
use crate::bases::b58::{bytes_to_str_base58, ToBase58};
use crate::bases::*;
use crate::rand::UnspecifiedRandError;
use crate::seeds::Seed32;
use base64;
use ring::signature::{Ed25519KeyPair as RingKeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::de::{Deserializer, Error, SeqAccess, Visitor};
use serde::ser::{SerializeTuple, Serializer};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use unwrap::unwrap;
use zeroize::Zeroize;

/// Maximal size of a public key in bytes
pub static PUBKEY_SIZE_IN_BYTES: &usize = &32;
/// Minimal size of a public key in bytes
pub static PUBKEY_MIN_SIZE_IN_BYTES: &usize = &31;
/// Size of a signature in bytes
pub static SIG_SIZE_IN_BYTES: &usize = &64;

/// Store a ed25519 signature.
#[derive(Clone, Copy)]
pub struct Signature(pub [u8; 64]);

impl Hash for Signature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(self.0.len())?;
        for elem in &self.0[..] {
            seq.serialize_element(elem)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Signature, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor {
            element: PhantomData<u8>,
        }

        impl<'de> Visitor<'de> for ArrayVisitor {
            type Value = Signature;

            #[cfg_attr(tarpaulin, skip)]
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str(concat!("an array of length ", 64))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Signature, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = [0u8; 64];
                for (i, byte) in arr.iter_mut().take(64).enumerate() {
                    *byte = seq
                        .next_element()?
                        .ok_or_else(|| Error::invalid_length(i, &self))?;
                }
                Ok(Signature(arr))
            }
        }

        let visitor: ArrayVisitor = ArrayVisitor {
            element: PhantomData,
        };
        deserializer.deserialize_tuple(64, visitor)
    }
}

impl super::Signature for Signature {
    #[inline]
    fn from_base64(base64_data: &str) -> Result<Signature, BaseConvertionError> {
        Ok(Signature(b64::str_base64_to64bytes(base64_data)?))
    }

    fn to_bytes_vector(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_base64(&self) -> String {
        base64::encode(&self.0[..]) // need to take a slice for required trait `AsRef<[u8]>`
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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
#[derive(Copy, Clone, Deserialize, PartialEq, Eq, Hash, Serialize)]
pub struct PublicKey {
    datas: [u8; 32],
    len: usize,
}

impl Default for PublicKey {
    fn default() -> Self {
        PublicKey {
            datas: [0u8; 32],
            len: 32,
        }
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.datas[..self.len]
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = PubkeyFromBytesError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() > *PUBKEY_SIZE_IN_BYTES || bytes.len() < *PUBKEY_MIN_SIZE_IN_BYTES {
            Err(PubkeyFromBytesError::InvalidBytesLen {
                expected: *PUBKEY_SIZE_IN_BYTES,
                found: bytes.len(),
            })
        } else {
            let mut u8_array = [0; 32];
            u8_array[..bytes.len()].copy_from_slice(&bytes);
            Ok(PublicKey {
                datas: u8_array,
                len: bytes.len(),
            })
        }
    }
}

impl ToBase58 for PublicKey {
    fn to_base58(&self) -> String {
        bytes_to_str_base58(self.as_ref())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", bytes_to_str_base58(self.as_ref()))
    }
}

impl Debug for PublicKey {
    // PublicKey { DNann1L... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "PublicKey {{ {} }}", self)
    }
}

impl super::PublicKey for PublicKey {
    type Signature = Signature;

    #[inline]
    fn from_base58(base58_data: &str) -> Result<Self, BaseConvertionError> {
        let (datas, len) = b58::str_base58_to_32bytes(base58_data)?;
        if len < *PUBKEY_MIN_SIZE_IN_BYTES {
            Err(BaseConvertionError::InvalidLength {
                expected: *PUBKEY_SIZE_IN_BYTES,
                found: len,
            })
        } else {
            Ok(PublicKey { datas, len })
        }
    }

    fn to_bytes_vector(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError> {
        Ok(UnparsedPublicKey::new(&ED25519, self.as_ref())
            .verify(message, &signature.0)
            .map_err(|_| SigError::InvalidSig)?)
    }
}

#[inline]
fn get_ring_ed25519_pubkey(ring_key_pair: &RingKeyPair) -> PublicKey {
    let ring_pubkey: <RingKeyPair as KeyPair>::PublicKey = *ring_key_pair.public_key();
    unwrap!(PublicKey::try_from(ring_pubkey.as_ref()))
}

/// Store a ed25519 cryptographic signator
#[derive(Debug)]
pub struct Signator(RingKeyPair);

impl super::Signator for Signator {
    type Signature = Signature;
    type PublicKey = PublicKey;

    fn public_key(&self) -> Self::PublicKey {
        get_ring_ed25519_pubkey(&self.0)
    }
    fn sign(&self, message: &[u8]) -> Self::Signature {
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(self.0.sign(message).as_ref());
        Signature(sig_bytes)
    }
}

/// Store a ed25519 cryptographic key pair (`PublicKey` + `PrivateKey`)
#[derive(Debug, Clone, Eq)]
pub struct Ed25519KeyPair {
    /// Store a Ed25519 public key.
    pub pubkey: PublicKey,
    /// Store a seed of 32 bytes.
    pub seed: Seed32,
}

impl Display for Ed25519KeyPair {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, hidden)", self.pubkey.to_base58())
    }
}

impl PartialEq<Ed25519KeyPair> for Ed25519KeyPair {
    fn eq(&self, other: &Ed25519KeyPair) -> bool {
        self.pubkey.eq(&other.pubkey) && self.seed.eq(&other.seed)
    }
}

impl super::KeyPair for Ed25519KeyPair {
    type Signator = Signator;

    fn generate_signator(&self) -> Result<Self::Signator, super::SignError> {
        Ok(Signator(
            RingKeyPair::from_seed_and_public_key(self.seed.as_ref(), self.pubkey.as_ref())
                .map_err(|_| super::SignError::CorruptedKeyPair)?,
        ))
    }

    fn public_key(&self) -> PublicKey {
        self.pubkey
    }

    fn seed(&self) -> &Seed32 {
        &self.seed
    }

    fn verify(
        &self,
        message: &[u8],
        signature: &<Self::Signator as super::Signator>::Signature,
    ) -> Result<(), SigError> {
        self.public_key().verify(message, signature)
    }
}

impl Ed25519KeyPair {
    /// Generate random keypair
    pub fn generate_random() -> Result<Self, UnspecifiedRandError> {
        Ok(KeyPairFromSeed32Generator::generate(Seed32::random()?))
    }
}

/// Keypair generator with seed
#[derive(Debug, Copy, Clone)]
pub struct KeyPairFromSeed32Generator {}

impl KeyPairFromSeed32Generator {
    /// Create a keypair based on a given seed.
    ///
    /// The [`PublicKey`](struct.PublicKey.html) will be able to verify messaged signed with
    /// the [`PrivateKey`](struct.PrivateKey.html).
    pub fn generate(seed: Seed32) -> Ed25519KeyPair {
        let ring_key_pair = RingKeyPair::from_seed_unchecked(seed.as_ref())
            .expect("dev error: fail to generate ed25519 keypair.");
        Ed25519KeyPair {
            pubkey: get_ring_ed25519_pubkey(&ring_key_pair),
            seed,
        }
    }
}

#[derive(Zeroize)]
#[zeroize(drop)]
/// Salted password
pub struct SaltedPassword {
    salt: String,
    password: String,
}

impl SaltedPassword {
    /// Create new salted password
    pub fn new(salt: String, password: String) -> SaltedPassword {
        SaltedPassword { salt, password }
    }
}

/// Keypair generator with given parameters for `scrypt` keypair function.
#[derive(Copy, Clone)]
pub struct KeyPairFromSaltedPasswordGenerator {
    scrypt_params: scrypt::ScryptParams,
}

impl KeyPairFromSaltedPasswordGenerator {
    /// Create a `KeyPairGenerator` with default arguments `(log_n: 12, r: 16, p: 1)`
    pub fn with_default_parameters() -> KeyPairFromSaltedPasswordGenerator {
        KeyPairFromSaltedPasswordGenerator {
            scrypt_params: scrypt::ScryptParams::new(12, 16, 1)
                .expect("dev error: invalid default scrypt params"),
        }
    }

    /// Create a `KeyPairFromSaltedPasswordGenerator` with given arguments.
    ///
    /// # Arguments
    ///
    /// - log_n - The log2 of the Scrypt parameter N
    /// - r - The Scrypt parameter r
    /// - p - The Scrypt parameter p
    pub fn with_parameters(
        log_n: u8,
        r: u32,
        p: u32,
    ) -> Result<KeyPairFromSaltedPasswordGenerator, scrypt::errors::InvalidParams> {
        Ok(KeyPairFromSaltedPasswordGenerator {
            scrypt_params: scrypt::ScryptParams::new(log_n, r, p)?,
        })
    }

    /// Create a seed based on a given password and salt.
    pub fn generate_seed(&self, password: &[u8], salt: &[u8]) -> Seed32 {
        let mut seed = [0u8; 32];

        scrypt::scrypt(password, salt, &self.scrypt_params, &mut seed)
            .expect("dev error: invalid seed len");

        Seed32::new(seed)
    }

    /// Create a keypair based on a given password and salt.
    ///
    /// The [`PublicKey`](struct.PublicKey.html) will be able to verify messaged signed with
    /// the [`PrivateKey`](struct.PrivateKey.html).
    pub fn generate(&self, salted_password: SaltedPassword) -> Ed25519KeyPair {
        // Generate seed from tuple (password + salt)
        let seed = self.generate_seed(
            salted_password.password.as_bytes(),
            salted_password.salt.as_bytes(),
        );
        // Generate keypair from seed
        KeyPairFromSeed32Generator::generate(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{KeyPair, Sig, Signator, Signature};
    use crate::seeds::Seed32;
    use bincode;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn base58_seed() {
        let seed58 = "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV";

        // Test base58 encoding/decoding (loop for every bytes)
        let seed = Seed32::from_base58(seed58).expect("fail to parser seed !");
        assert_eq!(seed.to_base58(), seed58);

        // Test seed display and debug
        assert_eq!(
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV".to_owned(),
            format!("{}", seed)
        );
        assert_eq!(
            "Seed32 { DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV }".to_owned(),
            format!("{:?}", seed)
        );

        // Test seed equality
        let same_seed = seed.clone();
        let other_seed = Seed32::default();
        assert!(seed.eq(&same_seed));
        assert!(!seed.eq(&other_seed));

        // Test seed parsing
        assert_eq!(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLVgth",).unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 35,
                expected: 32
            }
        );
        /*assert_eq!(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd",).unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 31,
                expected: 32
            }
        );*/
        assert_eq!(
            Seed32::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd<<").unwrap_err(),
            BaseConvertionError::InvalidCharacter {
                character: '<',
                offset: 42
            }
        );
        assert_eq!(
            Seed32::from_base58(
                "\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 "
            )
            .unwrap_err(),
            BaseConvertionError::InvalidLength {
                expected: 32,
                found: 161
            }
        );
    }

    #[test]
    fn base58_public_key() {
        let public58 = "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV";
        let public_key = unwrap!(super::PublicKey::from_base58(public58));

        // Test base58 encoding/decoding (loop for every bytes)
        assert_eq!(public_key.to_base58(), public58);
        let public_raw = unwrap!(b58::str_base58_to_32bytes(public58));
        assert_eq!(public_raw.0.to_vec(), public_key.to_bytes_vector());
        for (key, raw) in public_key.as_ref().iter().zip(public_raw.0.iter()) {
            assert_eq!(key, raw);
        }

        // Test pubkey debug
        assert_eq!(
            "PublicKey { DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV }".to_owned(),
            format!("{:?}", public_key)
        );

        // Test pubkey with 43 characters
        let pubkey43 =
            super::PublicKey::from_base58("2nV7Dv4nhTJ9dZUvRJpL34vFP9b2BkDjKWv9iBW2JaR").unwrap();
        println!("pubkey43={:?}", pubkey43.as_ref());
        assert_eq!(
            format!("{:?}", pubkey43),
            "PublicKey { 2nV7Dv4nhTJ9dZUvRJpL34vFP9b2BkDjKWv9iBW2JaR }".to_owned(),
        );
        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd<<")
                .unwrap_err(),
            BaseConvertionError::InvalidCharacter {
                character: '<',
                offset: 42
            }
        );
        assert_eq!(
            super::PublicKey::from_base58(
                "\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV\
                 "
            )
            .unwrap_err(),
            BaseConvertionError::InvalidLength {
                expected: 32,
                found: 161
            }
        );
    }

    #[test]
    fn base64_signature() {
        let signature64 = "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
                           MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==";
        let signature = unwrap!(super::Signature::from_base64(signature64));

        // Test signature base64 encoding/decoding (loop for every bytes)
        assert_eq!(signature.to_base64(), signature64);
        let signature_raw = unwrap!(base64::decode(signature64));
        for (sig, raw) in signature.0.iter().zip(signature_raw.iter()) {
            assert_eq!(sig, raw);
        }

        // Test signature display and debug
        assert_eq!(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==".to_owned(),
            format!("{}", signature)
        );
        assert_eq!(
            "Signature { 1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg== }".to_owned(),
            format!("{:?}", signature)
        );

        // Test signature hash
        let mut hasher = DefaultHasher::new();
        signature.hash(&mut hasher);
        let hash1 = hasher.finish();
        let mut hasher = DefaultHasher::new();
        let signature_copy = signature.clone();
        signature_copy.hash(&mut hasher);
        let hash2 = hasher.finish();
        assert_eq!(hash1, hash2);

        // Test signature serialization/deserialization
        let mut bin_sig = bincode::serialize(&signature).expect("Fail to serialize signature !");
        assert_eq!(*SIG_SIZE_IN_BYTES, bin_sig.len());
        assert_eq!(signature.to_bytes_vector(), bin_sig);
        assert_eq!(
            signature,
            bincode::deserialize(&bin_sig).expect("Fail to deserialize signature !"),
        );
        bin_sig.push(0); // add on byte to simulate invalid length
        bincode::deserialize::<Sig>(&bin_sig)
            .expect_err("Deserialization must be fail because length is invalid !");

        // Test signature parsing
        assert_eq!(
            super::Signature::from_base64("YmhlaW9iaHNlcGlvaGVvaXNlcGl2ZXBvdm5pc2U=").unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 29,
                expected: 64
            }
        );
        assert_eq!(
            super::Signature::from_base64(
                "YmhlaW9iaHNlcGlvaGVvaXNlcGl2ZXBvdm5pc2V2c2JlaW9idmVpb3Zqc\
                 2V2Z3BpaHNlamVwZ25qZXNqb2dwZWpnaW9zZXNkdnNic3JicmJyZGJyZGI=",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 86,
                expected: 64
            }
        );
        assert_eq!(
            super::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FGMM\
                 mQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAgdha<<",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidCharacter {
                character: '<',
                offset: 89
            }
        );
        assert_eq!(
            super::Signature::from_base64(
                "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
                 MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg===",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidBaseConverterLength,
        );
    }

    #[test]
    fn message_sign_verify() {
        let seed = unwrap!(Seed32::from_base58(
            "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV"
        ));

        let expected_signature = unwrap!(super::Signature::from_base64(
            "9ARKYkEAwp+kQ01rgvWUwJLchVLpZvHg3t/3H32XwWOoG119NiVCtfPSPtR4GDOeOz6Y+29drOLahqhzy+ciBw==",
        ));

        let message = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let signator = KeyPairFromSeed32Generator::generate(seed)
            .generate_signator()
            .expect("fail to generate signator !");
        let pubkey = signator.public_key();
        let sig = signator.sign(message.as_bytes());
        let wrong_sig = Signature([
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);

        assert_eq!(sig, expected_signature);
        assert_eq!(Ok(()), pubkey.verify(message.as_bytes(), &sig));
        assert_eq!(
            Err(SigError::InvalidSig),
            pubkey.verify(message.as_bytes(), &wrong_sig)
        );
    }

    #[test]
    fn keypair_generate() {
        let key_pair1 = KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            SaltedPassword::new(
                "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".to_owned(),
                "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".to_owned(),
            ),
        );

        assert_eq!(
            key_pair1.pubkey.to_string(),
            "7iMV3b6j2hSj5WtrfchfvxivS9swN3opDgxudeHq64fb"
        );

        let key_pair2 = KeyPairFromSaltedPasswordGenerator::with_parameters(12u8, 16, 1)
            .expect("fail to create KeyPairFromSaltedPasswordGenerator: invalid scrypt params.")
            .generate(SaltedPassword::new("toto".to_owned(), "toto".to_owned()));

        // Test signature display and debug
        assert_eq!(
            "(EA7Dsw39ShZg4SpURsrgMaMqrweJPUFPYHwZA8e92e3D, hidden)".to_owned(),
            format!("{}", key_pair2)
        );

        // Test key_pair equality
        let same_key_pair = key_pair2.clone();
        let other_key_pair = KeyPairFromSeed32Generator::generate(Seed32::new([
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]));
        assert!(key_pair2.eq(&same_key_pair));
        assert!(!key_pair2.eq(&other_key_pair));
    }

    #[test]
    fn keypair_generate_sign_and_verify() {
        let keypair = KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            SaltedPassword::new("password".to_owned(), "salt".to_owned()),
        );

        let message = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let sig = keypair
            .generate_signator()
            .expect("fail to gen signator")
            .sign(message.as_bytes());
        assert!(keypair.verify(message.as_bytes(), &sig).is_ok());
    }

    /*#[test]
    fn test_tmp() {
        let message = "InnerHash: A9697A9954EA447BBDC88D1B22AA8B60B2D11986DE806319C1A5AAFEB348C213\nNonce: 10300000043648\n";

        let pubkey = unwrap!(PublicKey::from_base58("8kXygUHh1vLjmcRzXVM86t38EL8dfFJgfBeHmkaWLamu"));
        let sig: super::Signature = unwrap!(Signature::from_base64(
            "XDIvgPbJK02ZfMwrhrtNFmMVGhqazDBhnxPBvMXLsDgPbnh28NbUbOYIRHrsZlo/frAv/Oh0OUOQZD3JpSf8DQ=="
        ));

        let pubkey_hex = hex::encode(&pubkey.0[..]);
        println!("{}", pubkey_hex);
        let sig_hex = hex::encode(&sig.0[..]);
        println!("{}", sig_hex);

        assert!(pubkey.verify(message.as_bytes(), &sig).is_ok());
    }*/
}

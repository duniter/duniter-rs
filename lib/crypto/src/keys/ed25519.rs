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

use super::{PrivateKey as PrivateKeyMethods, PublicKey as PublicKeyMethods};
use crate::bases::*;
use base58::ToBase58;
use base64;
use crypto;
use rand::{thread_rng, Rng};
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeTuple, Serializer};
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Size of a public key in bytes
pub static PUBKEY_SIZE_IN_BYTES: &'static usize = &32;
/// Size of a signature in bytes
pub static SIG_SIZE_IN_BYTES: &'static usize = &64;

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
pub struct PublicKey(pub [u8; 32]);

impl ToBase58 for PublicKey {
    fn to_base58(&self) -> String {
        self.0.to_base58()
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for PublicKey {
    // PublicKey { DNann1L... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "PublicKey {{ {} }}", self)
    }
}

use crate::keys::SigError;

impl super::PublicKey for PublicKey {
    type Signature = Signature;

    #[inline]
    fn from_base58(base58_data: &str) -> Result<Self, BaseConvertionError> {
        Ok(PublicKey(b58::str_base58_to_32bytes(base58_data)?))
    }

    fn to_bytes_vector(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError> {
        if crypto::ed25519::verify(message, &self.0, &signature.0) {
            Ok(())
        } else {
            Err(SigError::InvalidSig)
        }
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_base58())
    }
}

impl Debug for PrivateKey {
    // PrivateKey { 468Q1XtT... }
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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

    #[inline]
    fn from_base58(base58_data: &str) -> Result<Self, BaseConvertionError> {
        Ok(PrivateKey(b58::str_base58_to_64bytes(base58_data)?))
    }

    /// Sign a message with this private key.
    fn sign(&self, message: &[u8]) -> Self::Signature {
        Signature(crypto::ed25519::signature(message, &self.0))
    }
}

/// Store a ed25519 cryptographic key pair (`PublicKey` + `PrivateKey`)
#[derive(Debug, Copy, Clone, Eq)]
pub struct KeyPair {
    /// Store a Ed25519 public key.
    pub pubkey: PublicKey,
    /// Store a Ed25519 private key.
    pub privkey: PrivateKey,
}

impl Display for KeyPair {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, hidden)", self.pubkey.to_base58())
    }
}

impl PartialEq<KeyPair> for KeyPair {
    fn eq(&self, other: &KeyPair) -> bool {
        self.pubkey.eq(&other.pubkey) && self.privkey.eq(&other.privkey)
    }
}

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

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> Result<(), SigError> {
        self.public_key().verify(message, signature)
    }
}

impl KeyPair {
    /// Generate random keypair
    pub fn generate_random() -> Self {
        KeyPairFromSeedGenerator::generate(&thread_rng().gen::<[u8; 32]>())
    }
}

/// Keypair generator with seed
#[derive(Debug, Copy, Clone)]
pub struct KeyPairFromSeedGenerator {}

impl KeyPairFromSeedGenerator {
    /// Create a keypair based on a given seed.
    ///
    /// The [`PublicKey`](struct.PublicKey.html) will be able to verify messaged signed with
    /// the [`PrivateKey`](struct.PrivateKey.html).
    pub fn generate(seed: &[u8; 32]) -> KeyPair {
        let (private, public) = crypto::ed25519::keypair(seed);
        KeyPair {
            pubkey: PublicKey(public),
            privkey: PrivateKey(private),
        }
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

    /// Create a seed based on a given password and salt.
    pub fn generate_seed(&self, password: &[u8], salt: &[u8]) -> [u8; 32] {
        let mut seed = [0u8; 32];

        crypto::scrypt::scrypt(
            salt,
            password,
            &crypto::scrypt::ScryptParams::new(self.log_n, self.r, self.p),
            &mut seed,
        );

        seed
    }

    /// Create a keypair based on a given password and salt.
    ///
    /// The [`PublicKey`](struct.PublicKey.html) will be able to verify messaged signed with
    /// the [`PrivateKey`](struct.PrivateKey.html).
    pub fn generate(&self, password: &[u8], salt: &[u8]) -> KeyPair {
        let seed: [u8; 32] = self.generate_seed(password, salt);
        KeyPairFromSeedGenerator::generate(&seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{KeyPair, Sig, Signature};
    use base58::FromBase58;
    use bincode;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn base58_private_key() {
        let private58 =
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9r\
             qnXuW3iAfZACm7";
        let private_key = super::PrivateKey::from_base58(private58).unwrap();

        // Test base58 encoding/decoding (loop for every bytes)
        assert_eq!(private_key.to_base58(), private58);
        let private_raw = private58.from_base58().unwrap();
        for (key, raw) in private_key.0.iter().zip(private_raw.iter()) {
            assert_eq!(key, raw);
        }

        // Test privkey display and debug
        assert_eq!(
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7".to_owned(),
            format!("{}", private_key)
        );
        assert_eq!(
            "PrivateKey { 468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7 }".to_owned(),
            format!("{:?}", private_key)
        );

        // Test privkey equality
        let same_private_key = private_key.clone();
        let other_private_key = PrivateKey([
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ]);
        assert!(private_key.eq(&same_private_key));
        assert!(!private_key.eq(&other_private_key));

        // Test privkey parsing
        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iA\
                fZACm7djh",
            ).unwrap_err(),
            BaseConvertionError::InvalidLength { found: 67, expected: 64 }
        );
        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 53,
                expected: 64
            }
        );
        assert_eq!(
            super::PrivateKey::from_base58(
                "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9<<",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidCharacter {
                character: '<',
                offset: 73
            }
        );
        assert_eq!(
            super::PrivateKey::from_base58(
                "\
                 468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9\
                 468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9\
                 468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt5GiERP7ySs3wM8myLccbAAGejgMRC9\
                 ",
            )
            .unwrap_err(),
            BaseConvertionError::InvalidBaseConverterLength,
        );
    }

    #[test]
    fn base58_public_key() {
        let public58 = "DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV";
        let public_key = super::PublicKey::from_base58(public58).unwrap();

        // Test base58 encoding/decoding (loop for every bytes)
        assert_eq!(public_key.to_base58(), public58);
        let public_raw = public58.from_base58().unwrap();
        assert_eq!(public_raw, public_key.to_bytes_vector());
        for (key, raw) in public_key.0.iter().zip(public_raw.iter()) {
            assert_eq!(key, raw);
        }

        // Test pubkey debug
        assert_eq!(
            "PublicKey { DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV }".to_owned(),
            format!("{:?}", public_key)
        );

        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLVdjq")
                .unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 35,
                expected: 32
            }
        );
        assert_eq!(
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQd")
                .unwrap_err(),
            BaseConvertionError::InvalidLength {
                found: 31,
                expected: 32
            }
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
            BaseConvertionError::InvalidBaseConverterLength
        );
    }

    #[test]
    fn base64_signature() {
        let signature64 = "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
                           MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==";
        let signature = super::Signature::from_base64(signature64).unwrap();

        // Test signature base64 encoding/decoding (loop for every bytes)
        assert_eq!(signature.to_base64(), signature64);
        let signature_raw = base64::decode(signature64).unwrap();
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
        let pubkey =
            super::PublicKey::from_base58("DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV").unwrap();

        let prikey = super::PrivateKey::from_base58(
            "468Q1XtTq7h84NorZdWBZFJrGkB18CbmbHr9tkp9snt\
             5GiERP7ySs3wM8myLccbAAGejgMRC9rqnXuW3iAfZACm7",
        )
        .unwrap();

        let expected_signature = super::Signature::from_base64(
            "1eubHHbuNfilHMM0G2bI30iZzebQ2cQ1PC7uPAw08FG\
             MMmQCRerlF/3pc4sAcsnexsxBseA/3lY03KlONqJBAg==",
        )
        .unwrap();

        let message = "Version: 10
Type: Identity
Currency: duniter_unit_test_currency
Issuer: DNann1Lh55eZMEDXeYt59bzHbA3NJR46DeQYCS2qQdLV
UniqueID: tic
Timestamp: 0-E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855
";

        let sig = prikey.sign(message.as_bytes());
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

    #[cfg(unix)]
    #[test]
    fn keypair_generate() {
        let key_pair1 = KeyPairFromSaltedPasswordGenerator::with_default_parameters().generate(
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV".as_bytes(),
            "JhxtHB7UcsDbA9wMSyMKXUzBZUQvqVyB32KwzS9SWoLkjrUhHV_".as_bytes(),
        );

        assert_eq!(
            key_pair1.pubkey.to_string(),
            "7iMV3b6j2hSj5WtrfchfvxivS9swN3opDgxudeHq64fb"
        );

        let key_pair2 = KeyPairFromSaltedPasswordGenerator::with_parameters(12u8, 16, 1)
            .generate(b"toto", b"toto");

        // Test signature display and debug
        assert_eq!(
            "(EA7Dsw39ShZg4SpURsrgMaMqrweJPUFPYHwZA8e92e3D, hidden)".to_owned(),
            format!("{}", key_pair2)
        );

        // Test key_pair equality
        let same_key_pair = key_pair2.clone();
        let other_key_pair = KeyPairFromSeedGenerator::generate(&[
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        assert!(key_pair2.eq(&same_key_pair));
        assert!(!key_pair2.eq(&other_key_pair));
    }

    #[cfg(unix)]
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
        assert!(keypair.verify(message.as_bytes(), &sig).is_ok());
    }
}
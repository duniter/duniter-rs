//  Copyright (C) 2019  Eloïs SANCHEZ.
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

//! Manage cryptographic encryption operations.

mod chacha20_poly1305_aead;

use crate::agreement::{SharedSecret, SharedSecretLen};
use crate::Result;
use std::io::{BufWriter, Read, Write};

/// Encryption algorithm
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncryptAlgo {
    /// ChaCha20 stream cipher use the Poly1305 authenticator with Associated Data (AEAD) algorithm (see https://tools.ietf.org/html/rfc7539).
    Chacha20Poly1305Aead,
}

impl Default for EncryptAlgo {
    fn default() -> Self {
        Self::Chacha20Poly1305Aead
    }
}

impl EncryptAlgo {
    pub(crate) fn shared_secret_len(self) -> SharedSecretLen {
        match self {
            Self::Chacha20Poly1305Aead => SharedSecretLen::B48,
        }
    }
}

#[derive(Clone, Debug)]
pub enum EncryptAlgoWithSecretKey {
    Chacha20Poly1305Aead(chacha20_poly1305_aead::SecretKey),
}

impl EncryptAlgoWithSecretKey {
    pub fn build(encrypt_algo: EncryptAlgo, shared_secret: SharedSecret) -> Self {
        match encrypt_algo {
            EncryptAlgo::Chacha20Poly1305Aead => {
                if let SharedSecret::B48(seed) = shared_secret {
                    Self::Chacha20Poly1305Aead(chacha20_poly1305_aead::SecretKey::new(&seed))
                } else {
                    panic!("dev error: EncryptAlgo::Chacha20Poly1305Aead must request shared secret of 48 bytes !")
                }
            }
        }
    }
}

#[inline]
pub(crate) fn decrypt<W: Write>(
    encrypted_datas: &[u8],
    algo_with_secret_key: &EncryptAlgoWithSecretKey,
    writer: &mut BufWriter<W>,
) -> Result<()> {
    match algo_with_secret_key {
        EncryptAlgoWithSecretKey::Chacha20Poly1305Aead(secret_key) => {
            chacha20_poly1305_aead::decrypt(encrypted_datas, secret_key, writer)
        }
    }
}

/// Encrypt datas
#[inline]
pub(crate) fn encrypt<R: Read, W: Write>(
    reader: &mut R,
    algo_with_secret_key: &EncryptAlgoWithSecretKey,
    writer: &mut BufWriter<W>,
) -> Result<()> {
    match algo_with_secret_key {
        EncryptAlgoWithSecretKey::Chacha20Poly1305Aead(secret_key) => {
            chacha20_poly1305_aead::encrypt(reader, secret_key, writer)
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::seeds::{tests::random_seed_48, Seed32, Seed48};

    pub fn gen_random_encrypt_algo_with_secret() -> EncryptAlgoWithSecretKey {
        let random_shared_secret = SharedSecret::B48(random_seed_48());
        EncryptAlgoWithSecretKey::build(EncryptAlgo::Chacha20Poly1305Aead, random_shared_secret)
    }

    #[test]
    fn test_default() {
        assert_eq!(EncryptAlgo::Chacha20Poly1305Aead, EncryptAlgo::default());
    }

    #[test]
    #[should_panic(
        expected = "dev error: EncryptAlgo::Chacha20Poly1305Aead must request shared secret of 48 bytes !"
    )]
    fn test_encryption_with_wrong_shared_secret_len() {
        let shared_secret = SharedSecret::B32(Seed32::new([
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]));
        EncryptAlgoWithSecretKey::build(EncryptAlgo::Chacha20Poly1305Aead, shared_secret);
    }

    #[test]
    fn test_encryption_ok() -> Result<()> {
        let datas = b"My secret datas".to_vec();

        let shared_secret = SharedSecret::B48(Seed48::new([
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47,
        ]));
        let encrypt_algo_with_secret_key =
            EncryptAlgoWithSecretKey::build(EncryptAlgo::Chacha20Poly1305Aead, shared_secret);

        let mut encrypted_datas = BufWriter::new(Vec::with_capacity(datas.len()));

        encrypt(
            &mut &datas[..],
            &encrypt_algo_with_secret_key,
            &mut encrypted_datas,
        )?;
        let encrypted_datas = encrypted_datas
            .into_inner()
            .expect("fail to flush encrypt buffer");

        let mut decrypted_datas = BufWriter::new(Vec::with_capacity(datas.len()));
        decrypt(
            &encrypted_datas,
            &encrypt_algo_with_secret_key,
            &mut decrypted_datas,
        )?;
        let decrypted_datas = decrypted_datas
            .into_inner()
            .expect("fail to flush decrypt buffer");

        println!("encrypted_datas={:?}", encrypted_datas);
        println!("decrypted_datas={:?}", decrypted_datas);

        assert_eq!(datas, decrypted_datas);

        Ok(())
    }
}

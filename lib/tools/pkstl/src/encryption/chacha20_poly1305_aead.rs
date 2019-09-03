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

//! Manage cryptographic encryption operations with Chacha20Poly1305Aead algorithm.

use crate::seeds::Seed48;
use crate::{Error, Result};
use clear_on_drop::clear::Clear;
use std::io::{BufWriter, Read, Write};

const CHACHA20_TAG_SIZE: usize = 16;

#[derive(Clone, Debug, Default)]
/// Secret key used for encryption algo
pub struct SecretKey {
    key: [u8; 32],
    nonce: [u8; 12],
    aad: [u8; 4],
}

impl Drop for SecretKey {
    #[inline]
    fn drop(&mut self) {
        <[u8; 32] as Clear>::clear(&mut self.key);
        <[u8; 12] as Clear>::clear(&mut self.nonce);
        <[u8; 4] as Clear>::clear(&mut self.aad);
    }
}

impl SecretKey {
    /// Create new secret key
    pub fn new(seed: &Seed48) -> SecretKey {
        let mut secret_key = SecretKey::default();

        secret_key.key.copy_from_slice(&seed.as_ref()[0..32]);
        secret_key.nonce.copy_from_slice(&seed.as_ref()[32..44]);
        secret_key.aad.copy_from_slice(&seed.as_ref()[44..48]);

        secret_key
    }
}

/// Decrypt datas
pub fn decrypt<W: Write>(
    encrypted_datas: &[u8],
    secret_key: &SecretKey,
    writer: &mut BufWriter<W>,
) -> Result<()> {
    let payload_len = encrypted_datas.len() - CHACHA20_TAG_SIZE;

    chacha20_poly1305_aead::decrypt(
        &secret_key.key,
        &secret_key.nonce,
        &secret_key.aad,
        &encrypted_datas[0..payload_len],
        &encrypted_datas[payload_len..],
        writer,
    )
    .map_err(Error::FailToDecryptDatas)?;

    Ok(())
}

/// Encrypt datas
pub fn encrypt<R: Read, W: Write>(
    reader: &mut R,
    secret_key: &SecretKey,
    writer: &mut BufWriter<W>,
) -> Result<()> {
    let tag = chacha20_poly1305_aead::encrypt_read(
        &secret_key.key,
        &secret_key.nonce,
        &secret_key.aad,
        reader,
        writer,
    )
    .map_err(Error::FailToEncryptDatas)?;

    writer
        .write(&tag.to_vec())
        .map_err(Error::FailToEncryptDatas)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::seeds::Seed48;

    #[test]
    fn test_encryption() -> Result<()> {
        let datas = b"My secret datas".to_vec();

        let secret_key = SecretKey::new(&Seed48::new([
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47,
        ]));

        let mut encrypted_datas = BufWriter::new(Vec::with_capacity(datas.len()));

        encrypt(&mut &datas[..], &secret_key, &mut encrypted_datas)?;
        let encrypted_datas = encrypted_datas
            .into_inner()
            .expect("fail to flush encrypt buffer");

        let mut decrypted_datas = BufWriter::new(Vec::with_capacity(datas.len()));
        decrypt(&encrypted_datas, &secret_key, &mut decrypted_datas)?;
        let decrypted_datas = decrypted_datas
            .into_inner()
            .expect("fail to flush decrypt buffer");

        println!("encrypted_datas={:?}", encrypted_datas);
        println!("decrypted_datas={:?}", decrypted_datas);

        assert_eq!(datas, decrypted_datas);

        Ok(())
    }
}

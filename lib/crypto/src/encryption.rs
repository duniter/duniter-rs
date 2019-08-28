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

//! Manage cryptographic encryption operations.

use crate::errors::CryptoError;
use crate::seeds::Seed48;
use clear_on_drop::clear::Clear;
use std::io::Read;

const CHACHA20_TAG_SIZE: usize = 16;

#[derive(Clone, Default)]
/// Secret key used for encryption algo
pub struct SecretKey {
    key: [u8; 32],
    nonce: [u8; 12],
    aad: [u8; 4],
}

impl Drop for SecretKey {
    #[inline]
    fn drop(&mut self) {
        self.key.clear();
        self.nonce.clear();
        self.aad.clear();
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
pub fn decrypt(encrypted_datas: &[u8], secret_key: &SecretKey) -> Result<Vec<u8>, CryptoError> {
    let payload_len = encrypted_datas.len() - CHACHA20_TAG_SIZE;

    let mut decrypted_datas = Vec::with_capacity(payload_len);

    chacha20_poly1305_aead::decrypt(
        &secret_key.key,
        &secret_key.nonce,
        &secret_key.aad,
        &encrypted_datas[0..payload_len],
        &encrypted_datas[payload_len..],
        &mut decrypted_datas,
    )
    .map_err(CryptoError::FailToDecryptDatas)?;

    Ok(decrypted_datas)
}

/// Encrypt datas
pub fn encrypt(datas: &[u8], secret_key: &SecretKey) -> Result<Vec<u8>, CryptoError> {
    let mut encrypted_datas = Vec::with_capacity(datas.len() + CHACHA20_TAG_SIZE);

    let tag = chacha20_poly1305_aead::encrypt(
        &secret_key.key,
        &secret_key.nonce,
        &secret_key.aad,
        datas,
        &mut encrypted_datas,
    )
    .map_err(CryptoError::FailToEncryptDatas)?;

    encrypted_datas.append(&mut tag.to_vec());

    Ok(encrypted_datas)
}

/// Encrypt datas from reader
pub fn encrypt_read<R: Read>(
    datas_max_size: usize,
    reader: &mut R,
    secret_key: &SecretKey,
) -> Result<Vec<u8>, CryptoError> {
    let mut encrypted_datas = Vec::with_capacity(datas_max_size + CHACHA20_TAG_SIZE);

    let tag = chacha20_poly1305_aead::encrypt_read(
        &secret_key.key,
        &secret_key.nonce,
        &secret_key.aad,
        reader,
        &mut encrypted_datas,
    )
    .map_err(CryptoError::FailToEncryptDatas)?;

    encrypted_datas.append(&mut tag.to_vec());

    Ok(encrypted_datas)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_encryption() -> Result<(), CryptoError> {
        let datas = b"My secret datas".to_vec();

        let secret_key = SecretKey::new(&Seed48::random());

        let encrypted_datas = encrypt(&datas, &secret_key)?;
        let decrypted_datas = decrypt(&encrypted_datas, &secret_key)?;

        println!("encrypted_datas={:?}", encrypted_datas);
        println!("decrypted_datas={:?}", decrypted_datas);

        assert_eq!(datas, decrypted_datas);

        let encrypted_datas = encrypt_read(datas.len(), &mut &datas[..], &secret_key)?;
        let decrypted_datas = decrypt(&encrypted_datas, &secret_key)?;

        println!("encrypted_datas={:?}", encrypted_datas);
        println!("decrypted_datas={:?}", decrypted_datas);

        assert_eq!(datas, decrypted_datas);

        Ok(())
    }
}

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

//! Manage cryptographic agreement operations.

use crate::errors::CryptoError;
use ring::{agreement, digest, pbkdf2, rand};
use std::num::NonZeroU32;

const SHARED_SECRET_LEN: usize = digest::SHA384_OUTPUT_LEN;
const ITERATIONS: u32 = 3;

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA384;

#[derive(Clone)]
/// Ephemeral public key used once to generate shared secret
pub struct EphemeralPublicKey(agreement::PublicKey);

impl AsRef<[u8]> for EphemeralPublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Ephemeral key pair used once to generate shared secret
pub struct EphemeralKeyPair {
    privkey: agreement::EphemeralPrivateKey,
    pubkey: EphemeralPublicKey,
}

impl EphemeralKeyPair {
    /// Generate ephemeral key pair
    pub fn generate() -> Result<Self, CryptoError> {
        let rng = rand::SystemRandom::new();
        let privkey = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
            .map_err(|_| CryptoError::FailToGenEphemerKeyPair)?;
        let pubkey = EphemeralPublicKey(
            privkey
                .compute_public_key()
                .map_err(|_| CryptoError::FailToGenEphemerPubKey)?,
        );

        Ok(EphemeralKeyPair { privkey, pubkey })
    }
    /// Get ephemeral public key
    pub fn public_key(&self) -> &EphemeralPublicKey {
        &self.pubkey
    }
    /// Compute shared secret
    pub fn compute_shared_secret(
        self,
        other_ephemeral_public_key: &EphemeralPublicKey,
        server: bool,
    ) -> Result<[u8; SHARED_SECRET_LEN], CryptoError> {
        let salt = if server {
            self.pubkey.as_ref()
        } else {
            other_ephemeral_public_key.as_ref()
        };

        agreement::agree_ephemeral(
            self.privkey,
            &agreement::UnparsedPublicKey::new(
                &agreement::X25519,
                other_ephemeral_public_key.as_ref(),
            ),
            CryptoError::FailToComputeAgreement,
            |key_material| Ok(derive(key_material, salt)),
        )
    }
}

fn derive(seed: &[u8], salt: &[u8]) -> [u8; SHARED_SECRET_LEN] {
    let mut store_credentials: [u8; SHARED_SECRET_LEN] = [0u8; SHARED_SECRET_LEN];
    pbkdf2::derive(
        PBKDF2_ALG,
        NonZeroU32::new(ITERATIONS).expect("ITERATIONS must be > 0"),
        salt,
        seed,
        &mut store_credentials,
    );
    store_credentials
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_exchange_dh() -> Result<(), CryptoError> {
        let ephemeral_kp_server = EphemeralKeyPair::generate()?;
        let ephemeral_kp_client = EphemeralKeyPair::generate()?;

        let ephemeral_pk_server = ephemeral_kp_server.public_key().clone();
        let ephemeral_pk_client = ephemeral_kp_client.public_key().clone();

        let shared_secret_server =
            ephemeral_kp_server.compute_shared_secret(&ephemeral_kp_client.public_key(), true)?;

        let shared_secret_client =
            ephemeral_kp_client.compute_shared_secret(&ephemeral_pk_server, false)?;

        assert_eq!(shared_secret_server.to_vec(), shared_secret_client.to_vec());

        println!("ephemeral_pk_server={:?}", ephemeral_pk_server.as_ref());
        println!("ephemeral_pk_client={:?}", ephemeral_pk_client.as_ref());
        println!("shared_secret_server={:?}", shared_secret_server.to_vec());
        println!("shared_secret_client={:?}", shared_secret_client.to_vec());
        //panic!();
        Ok(())
    }
}

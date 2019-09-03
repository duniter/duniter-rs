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

//! Manage cryptographic agreement operations.

use crate::seeds::{Seed32, Seed48, Seed64};
use crate::{Error, Result};
use ring::{agreement, pbkdf2, rand};
use std::num::NonZeroU32;

const ITERATIONS: u32 = 3;

#[derive(Clone, Copy, Debug)]
pub enum SharedSecretLen {
    B32,
    B48,
    B64,
}

impl SharedSecretLen {
    fn algo(self) -> pbkdf2::Algorithm {
        match self {
            Self::B32 => pbkdf2::PBKDF2_HMAC_SHA256,
            Self::B48 => pbkdf2::PBKDF2_HMAC_SHA384,
            Self::B64 => pbkdf2::PBKDF2_HMAC_SHA512,
        }
    }
}

pub enum SharedSecret {
    B32(Seed32),
    B48(Seed48),
    B64(Seed64),
}

impl AsMut<[u8]> for SharedSecret {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::B32(seed) => seed.as_mut(),
            Self::B48(seed) => seed.as_mut(),
            Self::B64(seed) => seed.as_mut(),
        }
    }
}

impl SharedSecret {
    fn new(len: SharedSecretLen) -> Self {
        match len {
            SharedSecretLen::B32 => SharedSecret::B32(Seed32::default()),
            SharedSecretLen::B48 => SharedSecret::B48(Seed48::default()),
            SharedSecretLen::B64 => SharedSecret::B64(Seed64::default()),
        }
    }
}

#[derive(Clone, Debug)]
/// Ephemeral public key used once to generate shared secret
pub struct EphemeralPublicKey(agreement::PublicKey);

impl AsRef<[u8]> for EphemeralPublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Ephemeral key pair used once to generate shared secret
#[derive(Debug)]
pub struct EphemeralKeyPair {
    privkey: agreement::EphemeralPrivateKey,
    pubkey: EphemeralPublicKey,
}

impl EphemeralKeyPair {
    /// Generate ephemeral key pair
    pub fn generate() -> Result<Self> {
        let rng = rand::SystemRandom::new();
        let privkey = agreement::EphemeralPrivateKey::generate(&agreement::X25519, &rng)
            .map_err(|_| Error::FailToGenEphemerKeyPair)?;
        let pubkey = EphemeralPublicKey(
            privkey
                .compute_public_key()
                .map_err(|_| Error::FailToGenEphemerPubKey)?,
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
        other_ephemeral_public_key: &[u8],
        shared_secret_len: SharedSecretLen,
    ) -> Result<SharedSecret> {
        let salt = if self.pubkey.as_ref() > other_ephemeral_public_key {
            self.pubkey.as_ref()
        } else {
            other_ephemeral_public_key
        };

        agreement::agree_ephemeral(
            self.privkey,
            &agreement::UnparsedPublicKey::new(&agreement::X25519, other_ephemeral_public_key),
            Error::FailToComputeAgreement,
            |key_material| Ok(derive(key_material, salt, shared_secret_len)),
        )
    }
}

fn derive(seed: &[u8], salt: &[u8], shared_secret_len: SharedSecretLen) -> SharedSecret {
    let mut shared_secret = SharedSecret::new(shared_secret_len);
    pbkdf2::derive(
        shared_secret_len.algo(),
        NonZeroU32::new(ITERATIONS).expect("ITERATIONS must be > 0"),
        salt,
        seed,
        shared_secret.as_mut(),
    );
    shared_secret
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_exchange_dh_shared_secret_48b() -> Result<()> {
        let ephemeral_kp_server = EphemeralKeyPair::generate()?;
        let ephemeral_kp_client = EphemeralKeyPair::generate()?;

        let ephemeral_pk_server = ephemeral_kp_server.public_key().clone();
        let ephemeral_pk_client = ephemeral_kp_client.public_key().clone();

        // Sharer secret of 48 bytes
        let mut shared_secret_server_48b = ephemeral_kp_server.compute_shared_secret(
            ephemeral_kp_client.public_key().as_ref(),
            SharedSecretLen::B48,
        )?;

        let mut shared_secret_client_48b = ephemeral_kp_client
            .compute_shared_secret(ephemeral_pk_server.as_ref(), SharedSecretLen::B48)?;

        assert_eq!(
            shared_secret_server_48b.as_mut().to_vec(),
            shared_secret_client_48b.as_mut().to_vec()
        );

        println!("ephemeral_pk_server={:?}", ephemeral_pk_server.as_ref());
        println!("ephemeral_pk_client={:?}", ephemeral_pk_client.as_ref());
        println!(
            "shared_secret_server={:?}",
            shared_secret_server_48b.as_mut()
        );
        println!(
            "shared_secret_client={:?}",
            shared_secret_client_48b.as_mut()
        );

        Ok(())
    }

    #[test]
    fn test_exchange_dh_shared_secret_64b() -> Result<()> {
        let ephemeral_kp_server = EphemeralKeyPair::generate()?;
        let ephemeral_kp_client = EphemeralKeyPair::generate()?;

        let ephemeral_pk_server = ephemeral_kp_server.public_key().clone();
        let ephemeral_pk_client = ephemeral_kp_client.public_key().clone();

        // Sharer secret of 64 bytes
        let mut shared_secret_server_64b = ephemeral_kp_server.compute_shared_secret(
            ephemeral_kp_client.public_key().as_ref(),
            SharedSecretLen::B64,
        )?;

        let mut shared_secret_client_64b = ephemeral_kp_client
            .compute_shared_secret(ephemeral_pk_server.as_ref(), SharedSecretLen::B64)?;

        assert_eq!(
            shared_secret_server_64b.as_mut().to_vec(),
            shared_secret_client_64b.as_mut().to_vec()
        );

        println!("ephemeral_pk_server={:?}", ephemeral_pk_server.as_ref());
        println!("ephemeral_pk_client={:?}", ephemeral_pk_client.as_ref());
        println!(
            "shared_secret_server={:?}",
            shared_secret_server_64b.as_mut()
        );
        println!(
            "shared_secret_client={:?}",
            shared_secret_client_64b.as_mut()
        );

        Ok(())
    }
}

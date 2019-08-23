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

//! Manage cryptographic operations for the DUP (DUniter Protocol).

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
    missing_docs,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![allow(non_camel_case_types)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

pub mod bases;
pub mod hashs;
pub mod keys;

#[cfg(test)]
mod tests {

    use super::*;
    use ring::signature::Ed25519KeyPair;
    use ring::signature::KeyPair;

    #[test]
    fn test_ring_gen_keypair() {
        let seed = [
            61u8, 245, 136, 162, 155, 50, 205, 43, 116, 15, 45, 84, 138, 54, 114, 214, 71, 213, 11,
            251, 135, 182, 202, 131, 48, 91, 166, 226, 40, 255, 251, 172,
        ];

        let legacy_key_pair = keys::ed25519::KeyPairFromSeedGenerator::generate(&seed);

        let ring_key_pair: Ed25519KeyPair =
            Ed25519KeyPair::from_seed_and_public_key(&seed, &legacy_key_pair.pubkey.0)
                .expect("fail to generate ring key pair !");

        let ring_pubkey: <Ed25519KeyPair as KeyPair>::PublicKey = *ring_key_pair.public_key();
        let mut ring_pubkey_bytes: [u8; 32] = [0u8; 32];
        ring_pubkey_bytes.copy_from_slice(ring_pubkey.as_ref());

        assert_eq!(legacy_key_pair.pubkey.0, ring_pubkey_bytes);

        println!(
            "ring pubkey={}",
            keys::ed25519::PublicKey(ring_pubkey_bytes)
        );

        //panic!()
    }
}

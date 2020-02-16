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

//! Dunitrust keypairs

pub mod cli;

use crate::constants;
use crate::errors::DursConfError;
use dup_crypto::keys::*;
use durs_module::{RequiredKeys, RequiredKeysContent};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Keypairs filled in by the user (via a file or by direct entry in the terminal).
pub struct DuniterKeyPairs {
    /// Keypair used by the node to sign its communications with other nodes. This keypair is mandatory, if it's not filled in, a random keypair is generated.
    pub network_keypair: KeyPairEnum,
    /// Keypair used to sign the blocks forged by this node. If this keypair is'nt filled in, the node will not calculate blocks.
    pub member_keypair: Option<KeyPairEnum>,
}

impl Serialize for DuniterKeyPairs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let member_seed = if let Some(ref member_keypair) = self.member_keypair {
            member_keypair.seed().to_string()
        } else {
            String::from("")
        };
        let member_pub = if let Some(ref member_keypair) = self.member_keypair {
            member_keypair.public_key().to_string()
        } else {
            String::from("")
        };
        let mut state = serializer.serialize_struct("DuniterKeyPairs", 4)?;
        state.serialize_field(
            "network_seed",
            &self.network_keypair.seed().to_string().as_str(),
        )?;
        state.serialize_field(
            "network_pub",
            &self.network_keypair.public_key().to_string().as_str(),
        )?;
        state.serialize_field("member_seed", member_seed.as_str())?;
        state.serialize_field("member_pub", member_pub.as_str())?;
        state.end()
    }
}

impl DuniterKeyPairs {
    /// Returns only the keys indicated as required
    pub fn get_required_keys_content(
        required_keys: RequiredKeys,
        keypairs: DuniterKeyPairs,
    ) -> RequiredKeysContent {
        match required_keys {
            RequiredKeys::MemberKeyPair => {
                RequiredKeysContent::MemberKeyPair(keypairs.member_keypair)
            }
            RequiredKeys::MemberPublicKey => {
                RequiredKeysContent::MemberPublicKey(if let Some(keys) = keypairs.member_keypair {
                    Some(keys.public_key())
                } else {
                    None
                })
            }
            RequiredKeys::NetworkKeyPair => {
                RequiredKeysContent::NetworkKeyPair(keypairs.network_keypair)
            }
            RequiredKeys::NetworkPublicKey => {
                RequiredKeysContent::NetworkPublicKey(keypairs.network_keypair.public_key())
            }
            RequiredKeys::None => RequiredKeysContent::None,
        }
    }
}

/// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
fn generate_random_keypair(algo: KeysAlgo) -> KeyPairEnum {
    match algo {
        KeysAlgo::Ed25519 => KeyPairEnum::Ed25519(
            ed25519::Ed25519KeyPair::generate_random().expect("unspecified rand error"),
        ),
        KeysAlgo::Schnorr => panic!("Schnorr algo not yet supported !"),
    }
}

/// Save keypairs in profile folder
// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
pub fn write_keypairs_file(
    file_path: &PathBuf,
    keypairs: &DuniterKeyPairs,
) -> Result<(), std::io::Error> {
    let mut f = File::create(file_path.as_path())?;
    f.write_all(
        serde_json::to_string_pretty(keypairs)
            .unwrap_or_else(|_| panic!(dbg!("Fatal error : fail to deserialize keypairs !")))
            .as_bytes(),
    )?;
    f.sync_all()?;
    Ok(())
}

/// Load keypairs from file
pub fn load_keypairs_from_file(
    profile_path: &PathBuf,
    keypairs_file_path: &Option<PathBuf>,
) -> Result<DuniterKeyPairs, DursConfError> {
    // Get KeyPairs
    let keypairs_path = if let Some(ref keypairs_file_path) = keypairs_file_path {
        keypairs_file_path.clone()
    } else {
        let mut keypairs_path = profile_path.clone();
        keypairs_path.push(constants::KEYPAIRS_FILENAME);
        keypairs_path
    };
    if keypairs_path.as_path().exists() {
        if let Ok(mut f) = File::open(keypairs_path.as_path()) {
            let mut contents = String::new();
            if f.read_to_string(&mut contents).is_ok() {
                let json_conf: serde_json::Value =
                    serde_json::from_str(&contents).expect("Conf: Fail to parse keypairs file !");

                if let Some(network_seed) = json_conf.get("network_seed") {
                    if let Some(network_pub) = json_conf.get("network_pub") {
                        let network_seed = network_seed
                            .as_str()
                            .expect("Conf: Fail to parse keypairs file !");
                        let network_pub = network_pub
                            .as_str()
                            .expect("Conf: Fail to parse keypairs file !");
                        let network_keypair = KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                            seed: Seed32::from_base58(network_seed)
                                .expect("conf : keypairs file : fail to parse network_seed !"),
                            pubkey: ed25519::PublicKey::from_base58(network_pub)
                                .expect("conf : keypairs file : fail to parse network_pub !"),
                        });

                        let member_keypair = if let Some(member_seed) = json_conf.get("member_seed")
                        {
                            if let Some(member_pub) = json_conf.get("member_pub") {
                                let member_seed = member_seed
                                    .as_str()
                                    .expect("Conf: Fail to parse keypairs file !");
                                let member_pub = member_pub
                                    .as_str()
                                    .expect("Conf: Fail to parse keypairs file !");
                                if member_seed.is_empty() || member_pub.is_empty() {
                                    None
                                } else {
                                    Some(KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                                        seed: Seed32::from_base58(member_seed).expect(
                                            "conf : keypairs file : fail to parse member_seed !",
                                        ),
                                        pubkey: ed25519::PublicKey::from_base58(member_pub).expect(
                                            "conf : keypairs file : fail to parse member_pub !",
                                        ),
                                    }))
                                }
                            } else {
                                panic!("Fatal error : keypairs file wrong format : no field member_pub !")
                            }
                        } else {
                            panic!(
                                "Fatal error : keypairs file wrong format : no field member_seed !"
                            )
                        };

                        // Return keypairs
                        Ok(DuniterKeyPairs {
                            network_keypair,
                            member_keypair,
                        })
                    } else {
                        panic!("Fatal error : keypairs file wrong format : no field salt !")
                    }
                } else {
                    panic!("Fatal error : keypairs file wrong format : no field password !")
                }
            } else {
                panic!("Fail to read keypairs file !");
            }
        } else {
            panic!("Fail to open keypairs file !");
        }
    } else {
        // Create keypairs file with random keypair
        let keypairs = DuniterKeyPairs {
            network_keypair: generate_random_keypair(KeysAlgo::Ed25519),
            member_keypair: None,
        };
        write_keypairs_file(&keypairs_path, &keypairs).unwrap_or_else(|_| {
            panic!(dbg!("Fatal error : fail to write default keypairs file !"))
        });
        Ok(keypairs)
    }
}

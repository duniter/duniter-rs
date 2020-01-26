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

//! Dunitrust keypairs cli commands

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use crate::*;
use std::io;

#[derive(Debug, Copy, Clone)]
/// Errors encountered by the wizard
pub enum WizardError {
    /// Canceled
    Canceled,

    /// Bad input
    BadInput,
}

impl From<std::io::Error> for WizardError {
    fn from(_e: std::io::Error) -> Self {
        WizardError::BadInput
    }
}

/// Modify network keys command
pub fn modify_network_keys(
    salt: String,
    password: String,
    mut key_pairs: DuniterKeyPairs,
) -> DuniterKeyPairs {
    let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
    key_pairs.network_keypair =
        KeyPairEnum::Ed25519(generator.generate(ed25519::SaltedPassword::new(salt, password)));
    key_pairs
}

/// Modify member keys command
pub fn modify_member_keys(
    salt: String,
    password: String,
    mut key_pairs: DuniterKeyPairs,
) -> DuniterKeyPairs {
    let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
    key_pairs.member_keypair = Some(KeyPairEnum::Ed25519(
        generator.generate(ed25519::SaltedPassword::new(salt, password)),
    ));
    key_pairs
}

/// Ask user for confirmation and Clear keys command
pub fn clear_keys(network: bool, member: bool, key_pairs: DuniterKeyPairs) -> DuniterKeyPairs {
    inner_clear_keys(
        if network {
            if let Ok("y") = question_prompt("Clear your network keypair?", &["y", "n"]) {
                println!("Generating a new network keypair!");
                true
            } else {
                false
            }
        } else {
            false
        },
        if member {
            if let Ok("y") = question_prompt("Clear your member keypair?", &["y", "n"]) {
                println!("Deleting member keypair!");
                true
            } else {
                false
            }
        } else {
            false
        },
        key_pairs,
    )
}

/// Private function to Clear keys
fn inner_clear_keys(
    network: bool,
    member: bool,
    mut key_pairs: DuniterKeyPairs,
) -> DuniterKeyPairs {
    if network {
        key_pairs.network_keypair = super::generate_random_keypair(KeysAlgo::Ed25519);
    }
    if member {
        key_pairs.member_keypair = None
    }
    key_pairs
}

/// Show keys command
pub fn show_keys(key_pairs: DuniterKeyPairs) {
    println!("Network key: {}", key_pairs.network_keypair);
    match key_pairs.member_keypair {
        None => println!("No member key configured"),
        Some(key) => println!("Member key: {}", key),
    }
}

/// Save keys after a command run
pub fn save_keypairs(
    profile_path: PathBuf,
    keypairs_file_path: &Option<PathBuf>,
    key_pairs: DuniterKeyPairs,
) -> Result<(), std::io::Error> {
    let conf_keys_path: PathBuf = if let Some(keypairs_file_path) = keypairs_file_path {
        keypairs_file_path.to_path_buf()
    } else {
        let mut conf_keys_path = profile_path;
        conf_keys_path.push(crate::constants::KEYPAIRS_FILENAME);
        conf_keys_path
    };
    super::write_keypairs_file(&conf_keys_path, &key_pairs)?;
    Ok(())
}

fn question_prompt<'a>(question: &str, answers: &[&'a str]) -> Result<&'a str, WizardError> {
    let mut buf = String::new();

    println!("{} ({}):", question, answers.join("/"));
    let res = io::stdin().read_line(&mut buf);

    match res {
        Ok(_) => {
            let answer = answers.iter().find(|x| **x == buf.trim());
            match answer {
                Some(&value) => Ok(value),
                None => Err(WizardError::Canceled),
            }
        }
        Err(_) => Err(WizardError::Canceled),
    }
}

fn salt_password_prompt() -> Result<KeyPairEnum, WizardError> {
    let salt = rpassword::prompt_password_stdout("Salt: ")?;
    if !salt.is_empty() {
        let password = rpassword::prompt_password_stdout("Password: ")?;
        if !password.is_empty() {
            let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
            let key_pairs = KeyPairEnum::Ed25519(
                generator.generate(ed25519::SaltedPassword::new(salt, password)),
            );
            Ok(key_pairs)
        } else {
            Err(WizardError::BadInput)
        }
    } else {
        Err(WizardError::BadInput)
    }
}

/// The wizard key function
pub fn key_wizard(mut key_pairs: DuniterKeyPairs) -> Result<DuniterKeyPairs, WizardError> {
    let mut answer = question_prompt("Modify your network keypair?", &["y", "n"])?;
    if answer == "y" {
        key_pairs.network_keypair = salt_password_prompt()?;
    }

    answer = question_prompt("Modify your member keypair?", &["y", "n", "d"])?;
    if answer == "y" {
        key_pairs.member_keypair = Some(salt_password_prompt()?);
    } else if answer == "d" {
        println!("Deleting member keypair!");
        key_pairs.member_keypair = None;
    }

    Ok(key_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    use unwrap::unwrap;

    static BASE58_SEED_INIT: &'static str = "4iXXx5GgRkZ85BVPwn8vFXvztdXAAa5yB573ErcAnngA";
    static BASE58_PUB_INIT: &'static str = "otDgSpKvKAPPmE1MUYxc3UQ3RtEnKYz4iGD3BmwKPzM";
    //static SALT_INIT: &'static str = "initsalt";
    //static PASSWORD_INIT: &'static str = "initpassword";

    static BASE58_SEED_TEST: &'static str = "ELjDWGPyCGMuhr7R7H2aip6UJA9qLRepmK77pcD41UqQ";
    static BASE58_PUB_TEST: &'static str = "6sewkaNWyEMqkEa2PVRWrDb3hxWtjPdUSB1zXVCqhdWV";
    static SALT_TEST: &'static str = "testsalt";
    static PASSWORD_TEST: &'static str = "testpassword";

    #[test]
    fn test_modify_member_keys() {
        let key_pairs = DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }),
            member_keypair: None,
        };
        let result_key_pairs =
            modify_member_keys(SALT_TEST.to_owned(), PASSWORD_TEST.to_owned(), key_pairs);
        // We expect network key not to change
        assert_eq!(
            result_key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            result_key_pairs.network_keypair.seed().clone(),
            Seed32::from_base58(BASE58_SEED_INIT).expect("Wrong data in BASE58_SEED_INIT"),
        );

        // We expect member key to update as intended
        assert_eq!(
            unwrap!(
                result_key_pairs.member_keypair.clone(),
                "conf: member_keypair must have a value"
            )
            .public_key(),
            PubKey::Ed25519(unwrap!(
                ed25519::PublicKey::from_base58(BASE58_PUB_TEST),
                "Wrong data in BASE58_PUB_TEST"
            ))
        );
        assert_eq!(
            result_key_pairs
                .member_keypair
                .clone()
                .expect("conf: member_keypair must have a value")
                .seed()
                .clone(),
            Seed32::from_base58(BASE58_SEED_TEST).expect("Wrong data in BASE58_SEED_TEST"),
        );
    }

    #[test]
    fn test_modify_network_keys() {
        let key_pairs = DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }),
            member_keypair: None,
        };
        let result_key_pairs =
            modify_network_keys(SALT_TEST.to_owned(), PASSWORD_TEST.to_owned(), key_pairs);
        // We expect network key to update
        assert_eq!(
            result_key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_TEST)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            result_key_pairs.network_keypair.seed().clone(),
            Seed32::from_base58(BASE58_SEED_TEST).expect("Wrong data in BASE58_SEED_TEST")
        );
        // We expect member key not to change
        assert_eq!(result_key_pairs.member_keypair, None);
    }

    #[test]
    fn test_clear_network_keys() {
        let key_pairs = DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }),
            member_keypair: Some(KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            })),
        };
        let result_key_pairs = inner_clear_keys(true, false, key_pairs);
        // We expect network key to be reset to a new random key
        assert_ne!(
            result_key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_ne!(
            result_key_pairs.network_keypair.seed().clone(),
            unwrap!(
                Seed32::from_base58(BASE58_SEED_INIT),
                "Wrong data in BASE58_SEED_TEST"
            )
        );

        // We expect member key not to change
        assert_eq!(
            unwrap!(
                result_key_pairs.member_keypair.clone(),
                "conf: result_keypair must have a value"
            )
            .public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            result_key_pairs
                .member_keypair
                .clone()
                .expect("conf: result_keypair must have a value")
                .seed()
                .clone(),
            Seed32::from_base58(BASE58_SEED_INIT).expect("Wrong data in BASE58_SEED_TEST")
        );
    }

    #[test]
    fn test_clear_member_keys() {
        let key_pairs = DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }),
            member_keypair: Some(KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            })),
        };
        let result_key_pairs = inner_clear_keys(false, true, key_pairs);
        // We expect network key not to change
        assert_eq!(
            result_key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            result_key_pairs.network_keypair.seed().clone(),
            Seed32::from_base58(BASE58_SEED_INIT).expect("Wrong data in BASE58_SEED_TEST")
        );

        // We expect member key to change
        assert_eq!(result_key_pairs.member_keypair, None);
        assert_eq!(result_key_pairs.member_keypair, None);
    }
}

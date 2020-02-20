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
#[cfg(test)]
use mockall::*;
use std::io;

#[cfg_attr(test, automock)]
trait UserPasswordInput {
    fn get_password(&self, prompt: &str) -> std::io::Result<String>;
}

impl UserPasswordInput for std::io::Stdin {
    #[inline]
    fn get_password(&self, prompt: &str) -> std::io::Result<String> {
        Ok(rpassword::prompt_password_stdout(prompt)?)
    }
}

#[derive(Debug, Copy, Clone)]
/// Errors encountered by the user interaction
pub enum CliError {
    /// Canceled
    Canceled,

    /// Bad input
    BadInput,
}

impl From<std::io::Error> for CliError {
    fn from(_e: std::io::Error) -> Self {
        CliError::BadInput
    }
}

#[inline]
/// Modify network keys command
pub fn modify_network_keys(key_pairs: DuniterKeyPairs) -> Result<DuniterKeyPairs, CliError> {
    inner_modify_network_keys(std::io::stdin(), key_pairs)
}

/// Private function to modify network keys
fn inner_modify_network_keys<T: UserPasswordInput>(
    stdin: T,
    mut key_pairs: DuniterKeyPairs,
) -> Result<DuniterKeyPairs, CliError> {
    key_pairs.network_keypair = salt_password_prompt(stdin)?;
    Ok(key_pairs)
}

#[inline]
/// Modify member keys command
pub fn modify_member_keys(key_pairs: DuniterKeyPairs) -> Result<DuniterKeyPairs, CliError> {
    inner_modify_member_keys(std::io::stdin(), key_pairs)
}

/// Private function to modify network keys
fn inner_modify_member_keys<T: UserPasswordInput>(
    stdin: T,
    mut key_pairs: DuniterKeyPairs,
) -> Result<DuniterKeyPairs, CliError> {
    key_pairs.member_keypair = Some(salt_password_prompt(stdin)?);
    Ok(key_pairs)
}

/// Ask user for confirmation and Clear keys command
pub fn clear_keys(network: bool, member: bool, mut key_pairs: DuniterKeyPairs) -> DuniterKeyPairs {
    if network {
        if let Ok("y") = question_prompt("Clear your network keypair?", &["y", "n"]) {
            println!("Generating a new network keypair!");
            clear_network_key(&mut key_pairs);
        }
    }
    if member {
        if let Ok("y") = question_prompt("Clear your member keypair?", &["y", "n"]) {
            println!("Deleting member keypair!");
            clear_member_key(&mut key_pairs);
        }
    }
    key_pairs
}

#[inline]
/// Private function to Clear keys
fn clear_network_key(key_pairs: &mut DuniterKeyPairs) {
    key_pairs.network_keypair = super::generate_random_keypair(KeysAlgo::Ed25519);
}

#[inline]
/// Private function to Clear member key
fn clear_member_key(key_pairs: &mut DuniterKeyPairs) {
    key_pairs.member_keypair = None;
}

/// Show keys command
pub fn show_keys(key_pairs: DuniterKeyPairs) {
    show_network_keys(&key_pairs);
    show_member_keys(&key_pairs);
}

#[inline]
/// Show network keys
pub fn show_network_keys(key_pairs: &DuniterKeyPairs) {
    println!("Network key: {}", key_pairs.network_keypair);
}

#[inline]
/// Show member keys
pub fn show_member_keys(key_pairs: &DuniterKeyPairs) {
    match &key_pairs.member_keypair {
        None => println!("No member key configured"),
        Some(key) => println!("Member key: {}", key),
    }
}

/// Save keys after a command run
pub fn save_keypairs(
    profile_path: PathBuf,
    keypairs_file_path: &Option<PathBuf>,
    key_pairs: &DuniterKeyPairs,
) -> Result<(), std::io::Error> {
    let conf_keys_path: PathBuf = if let Some(keypairs_file_path) = keypairs_file_path {
        keypairs_file_path.to_path_buf()
    } else {
        let mut conf_keys_path = profile_path;
        conf_keys_path.push(crate::constants::KEYPAIRS_FILENAME);
        conf_keys_path
    };
    super::write_keypairs_file(&conf_keys_path, &key_pairs)
}

fn question_prompt<'a>(question: &str, answers: &[&'a str]) -> Result<&'a str, CliError> {
    let mut buf = String::new();

    println!("{} ({}):", question, answers.join("/"));
    let res = io::stdin().read_line(&mut buf);

    if res.is_ok() {
        answers
            .iter()
            .find(|x| **x == buf.trim())
            .copied()
            .ok_or(CliError::Canceled)
    } else {
        Err(CliError::Canceled)
    }
}

fn salt_password_prompt<T: UserPasswordInput>(stdin: T) -> Result<KeyPairEnum, CliError> {
    let salt = stdin.get_password("Salt: ")?;
    if !salt.is_empty() {
        let password = stdin.get_password("Password: ")?;
        if !password.is_empty() {
            let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
            let key_pairs = KeyPairEnum::Ed25519(
                generator.generate(ed25519::SaltedPassword::new(salt, password)),
            );
            Ok(key_pairs)
        } else {
            Err(CliError::BadInput)
        }
    } else {
        Err(CliError::BadInput)
    }
}

/// The wizard key function
pub fn key_wizard(mut key_pairs: DuniterKeyPairs) -> Result<DuniterKeyPairs, CliError> {
    let mut answer = question_prompt("Modify your network keypair?", &["y", "n"])?;
    if answer == "y" {
        key_pairs.network_keypair = salt_password_prompt(std::io::stdin())?;
    }

    answer = question_prompt("Modify your member keypair?", &["y", "n", "d"])?;
    if answer == "y" {
        key_pairs.member_keypair = Some(salt_password_prompt(std::io::stdin())?);
    } else if answer == "d" {
        println!("Deleting member keypair!");
        clear_member_key(&mut key_pairs);
    }

    Ok(key_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    use unwrap::unwrap;

    static BASE58_SEED_INIT: &str = "4iXXx5GgRkZ85BVPwn8vFXvztdXAAa5yB573ErcAnngA";
    static BASE58_PUB_INIT: &str = "otDgSpKvKAPPmE1MUYxc3UQ3RtEnKYz4iGD3BmwKPzM";

    static BASE58_SEED_TEST: &str = "ELjDWGPyCGMuhr7R7H2aip6UJA9qLRepmK77pcD41UqQ";
    static BASE58_PUB_TEST: &str = "6sewkaNWyEMqkEa2PVRWrDb3hxWtjPdUSB1zXVCqhdWV";
    static SALT_TEST: &str = "testsalt";
    static PASSWORD_TEST: &str = "testpassword";

    fn setup_user_password_input() -> MockUserPasswordInput {
        let mut stdin_mock = MockUserPasswordInput::new();
        stdin_mock
            .expect_get_password()
            .returning(|prompt| {
                if prompt.starts_with("Salt:") {
                    Ok(SALT_TEST.to_owned())
                } else if prompt.starts_with("Password:") {
                    Ok(PASSWORD_TEST.to_owned())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("should not be called with {}", prompt),
                    ))
                }
            })
            .times(2);
        stdin_mock
    }

    fn setup_keys(both_keys: bool) -> DuniterKeyPairs {
        let member_keypair = if both_keys {
            Some(KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }))
        } else {
            None
        };
        DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(ed25519::Ed25519KeyPair {
                seed: Seed32::from_base58(BASE58_SEED_INIT)
                    .expect("conf : keypairs file : fail to parse network_seed !"),
                pubkey: ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("conf : keypairs file : fail to parse network_pub !"),
            }),
            member_keypair,
        }
    }

    #[test]
    fn test_modify_member_keys() {
        let key_pairs = setup_keys(false);
        let stdin_mock = setup_user_password_input();
        let result_key_pairs =
            inner_modify_member_keys(stdin_mock, key_pairs).expect("Fail to read new member keys");
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
                .expect("conf: member_keypair must have a value")
                .seed()
                .clone(),
            Seed32::from_base58(BASE58_SEED_TEST).expect("Wrong data in BASE58_SEED_TEST"),
        );
    }

    #[test]
    fn test_modify_network_keys() {
        let key_pairs = setup_keys(false);
        let stdin_mock = setup_user_password_input();
        let result_key_pairs = inner_modify_network_keys(stdin_mock, key_pairs)
            .expect("Fail to read new network keys");
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
        let mut key_pairs = setup_keys(true);
        clear_network_key(&mut key_pairs);
        // We expect network key to be reset to a new random key
        assert_ne!(
            key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_ne!(
            key_pairs.network_keypair.seed().clone(),
            unwrap!(
                Seed32::from_base58(BASE58_SEED_INIT),
                "Wrong data in BASE58_SEED_TEST"
            )
        );

        // We expect member key not to change
        assert_eq!(
            unwrap!(
                key_pairs.member_keypair.clone(),
                "conf: keypair must have a value"
            )
            .public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            key_pairs
                .member_keypair
                .expect("conf: keypair must have a value")
                .seed()
                .clone(),
            Seed32::from_base58(BASE58_SEED_INIT).expect("Wrong data in BASE58_SEED_TEST")
        );
    }

    #[test]
    fn test_clear_member_keys() {
        let mut key_pairs = setup_keys(true);
        clear_member_key(&mut key_pairs);
        // We expect network key not to change
        assert_eq!(
            key_pairs.network_keypair.public_key(),
            PubKey::Ed25519(
                ed25519::PublicKey::from_base58(BASE58_PUB_INIT)
                    .expect("Wrong data in BASE58_PUB_TEST")
            )
        );
        assert_eq!(
            key_pairs.network_keypair.seed().clone(),
            Seed32::from_base58(BASE58_SEED_INIT).expect("Wrong data in BASE58_SEED_TEST")
        );

        // We expect member key to change
        assert_eq!(key_pairs.member_keypair, None);
    }
}

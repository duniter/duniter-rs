//  Copyright (C) 2018  The Duniter Project Developers.
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

//! Defined the few global types used by all modules,
//! as well as the DuniterModule trait that all modules must implement.

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate rpassword;

use std::io;
use *;

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
        KeyPairEnum::Ed25519(generator.generate(salt.as_bytes(), password.as_bytes()));
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
        generator.generate(salt.as_bytes(), password.as_bytes()),
    ));
    key_pairs
}

/// Clear keys command
pub fn clear_keys(network: bool, member: bool, mut key_pairs: DuniterKeyPairs) -> DuniterKeyPairs {
    if network {
        key_pairs.network_keypair = generate_random_keypair(KeysAlgo::Ed25519);
    }
    if member {
        key_pairs.member_keypair = None;
    }
    key_pairs
}

/// Show keys command
pub fn show_keys(key_pairs: DuniterKeyPairs) {
    println!("Network key : {}", key_pairs.network_keypair);
    match key_pairs.member_keypair {
        None => println!("No member key configured"),
        Some(key) => println!("Member key : {}", key),
    }
}

/// Save keys after a command run
pub fn save_keypairs(profile: &str, key_pairs: DuniterKeyPairs) {
    let conf_keys_path = keypairs_filepath(profile);
    write_keypairs_file(&conf_keys_path, &key_pairs).expect("could not write keypairs file");
}

fn question_prompt(question: &str, answers: Vec<String>) -> Result<String, WizardError> {
    let mut buf = String::new();

    println!("{} ({}) :", question, answers.join("/"));
    let res = io::stdin().read_line(&mut buf);

    match res {
        Ok(_) => {
            let answer = answers.into_iter().find(|x| x == buf.trim());
            match answer {
                Some(value) => Ok(value),
                None => Err(WizardError::Canceled),
            }
        }
        Err(_) => Err(WizardError::Canceled),
    }
}

fn salt_password_prompt() -> Result<KeyPairEnum, WizardError> {
    let salt = rpassword::prompt_password_stdout("? Salt: ")?;
    if !salt.is_empty() {
        let password = rpassword::prompt_password_stdout("? Password: ")?;
        if !password.is_empty() {
            let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
            let key_pairs = KeyPairEnum::Ed25519(generator.generate(
                salt.into_bytes().as_slice(),
                password.into_bytes().as_slice(),
            ));
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
    let mut answer = question_prompt(
        "? Modify your network keypair?",
        vec!["y".to_string(), "n".to_string()],
    )?;
    if answer == "y" {
        key_pairs.network_keypair = salt_password_prompt()?;
    }

    answer = question_prompt(
        "? Modify your member keypair?",
        vec!["y".to_string(), "n".to_string(), "d".to_string()],
    )?;
    if answer == "y" {
        key_pairs.member_keypair = Some(salt_password_prompt()?);
    } else if answer == "d" {
        println!("Deleting member keypair !");
        key_pairs.member_keypair = None;
    }

    Ok(key_pairs)
}

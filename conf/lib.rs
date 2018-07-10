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

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate duniter_crypto;
extern crate duniter_module;
extern crate rand;
extern crate serde;
use duniter_crypto::keys::*;
use duniter_module::{Currency, DuniterConf, ModuleId, RequiredKeys, RequiredKeysContent};
use rand::Rng;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

static USER_DATAS_FOLDER: &'static str = "durs-dev";

/// If no currency is specified by the user, is the currency will be chosen by default
pub static DEFAULT_CURRRENCY: &'static str = "g1";

#[derive(Debug, Clone)]
/// User request on global conf
pub enum ChangeGlobalConf {
    /// Change currency
    ChangeCurrency(Currency),
    /// Disable module
    DisableModule(ModuleId),
    /// Enable module
    EnableModule(ModuleId),
    /// None
    None(),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Duniter configuration v1
pub struct DuRsConfV1 {
    /// Currency
    pub currency: Currency,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Configuration of modules in json format (obtained from the conf.json file)
    pub modules: serde_json::Value,
    /// Disabled modules
    pub disabled: HashSet<ModuleId>,
    /// Enabled modules
    pub enabled: HashSet<ModuleId>,
}

impl Default for DuRsConfV1 {
    fn default() -> Self {
        DuRsConfV1 {
            currency: Currency::Str(String::from("g1")),
            my_node_id: generate_random_node_id(),
            modules: serde_json::Value::Null,
            disabled: HashSet::with_capacity(0),
            enabled: HashSet::with_capacity(0),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Duniter node configuration
pub enum DuRsConf {
    /// Duniter node configuration v1
    V1(DuRsConfV1),
    /// Duniter node configuration v2
    V2(),
}

impl Default for DuRsConf {
    fn default() -> Self {
        DuRsConf::V1(DuRsConfV1::default())
    }
}

impl DuniterConf for DuRsConf {
    fn version(&self) -> usize {
        match *self {
            DuRsConf::V1(ref _conf_v1) => 1,
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn currency(&self) -> Currency {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.currency.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn set_currency(&mut self, new_currency: Currency) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => conf_v1.currency = new_currency,
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn my_node_id(&self) -> u32 {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.my_node_id,
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn disable(&mut self, module: ModuleId) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => {
                conf_v1.disabled.insert(module.clone());
                conf_v1.enabled.remove(&module);
            }
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn enable(&mut self, module: ModuleId) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => {
                conf_v1.disabled.remove(&module);
                conf_v1.enabled.insert(module);
            }
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn disabled_modules(&self) -> HashSet<ModuleId> {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.disabled.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn enabled_modules(&self) -> HashSet<ModuleId> {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.enabled.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
    fn modules(&self) -> serde_json::Value {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.modules.clone(),
            _ => panic!("Fail to load duniter conf : conf version not supported !"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        let member_sec = if let Some(member_keypair) = self.member_keypair {
            member_keypair.private_key().to_string()
        } else {
            String::from("")
        };
        let member_pub = if let Some(member_keypair) = self.member_keypair {
            member_keypair.public_key().to_string()
        } else {
            String::from("")
        };
        let mut state = serializer.serialize_struct("DuniterKeyPairs", 4)?;
        state.serialize_field(
            "network_sec",
            &self.network_keypair.private_key().to_string().as_str(),
        )?;
        state.serialize_field(
            "network_pub",
            &self.network_keypair.public_key().to_string().as_str(),
        )?;
        state.serialize_field("member_sec", member_sec.as_str())?;
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
            RequiredKeys::MemberKeyPair() => {
                RequiredKeysContent::MemberKeyPair(keypairs.member_keypair)
            }
            RequiredKeys::MemberPublicKey() => {
                RequiredKeysContent::MemberPublicKey(if let Some(keys) = keypairs.member_keypair {
                    Some(keys.public_key())
                } else {
                    None
                })
            }
            RequiredKeys::NetworkKeyPair() => {
                RequiredKeysContent::NetworkKeyPair(keypairs.network_keypair)
            }
            RequiredKeys::NetworkPublicKey() => {
                RequiredKeysContent::NetworkPublicKey(keypairs.network_keypair.public_key())
            }
            RequiredKeys::None() => RequiredKeysContent::None(),
        }
    }
}

fn _use_json_macro() -> serde_json::Value {
    json!({})
}

fn generate_random_keypair(algo: KeysAlgo) -> KeyPairEnum {
    let mut rng = rand::thread_rng();
    match algo {
        KeysAlgo::Ed25519 => {
            let generator = ed25519::KeyPairFromSaltedPasswordGenerator::with_default_parameters();
            KeyPairEnum::Ed25519(generator.generate(&[rng.gen::<u8>(); 8], &[rng.gen::<u8>(); 8]))
        }
        KeysAlgo::Schnorr => panic!("Schnorr algo not yet supported !"),
    }
}

fn generate_random_node_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}

/// Return the user datas folder name
pub fn get_user_datas_folder() -> &'static str {
    USER_DATAS_FOLDER
}

/// Returns the path to the folder containing the currency datas of the running profile
pub fn datas_path(profile: &str, currency: &Currency) -> PathBuf {
    let mut datas_path = get_profile_path(profile);
    datas_path.push(currency.to_string());
    if !datas_path.as_path().exists() {
        fs::create_dir(datas_path.as_path()).expect("Impossible to create currency dir !");
    }
    datas_path
}

/// Returns the path to the folder containing the user data of the running profile
pub fn get_profile_path(profile: &str) -> PathBuf {
    // Define and create datas directory if not exist
    let mut profile_path = match env::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir !"),
    };
    profile_path.push(".config/");
    if !profile_path.as_path().exists() {
        fs::create_dir(profile_path.as_path()).expect("Impossible to create ~/.config dir !");
    }
    profile_path.push(USER_DATAS_FOLDER);
    if !profile_path.as_path().exists() {
        fs::create_dir(profile_path.as_path()).unwrap_or_else(|_| {
            panic!("Impossible to create ~/.config/{} dir !", USER_DATAS_FOLDER)
        });
    }
    profile_path.push(profile);
    if !profile_path.as_path().exists() {
        fs::create_dir(profile_path.as_path()).expect("Impossible to create your profile dir !");
    }
    profile_path
}

/// Load configuration.
pub fn load_conf(profile: &str) -> (DuRsConf, DuniterKeyPairs) {
    let mut profile_path = get_profile_path(profile);

    // Load conf
    let (conf, keypairs) = load_conf_at_path(profile, &profile_path);

    // Create currency dir
    profile_path.push(conf.currency().to_string());
    if !profile_path.as_path().exists() {
        fs::create_dir(profile_path.as_path()).expect("Impossible to create currency dir !");
    }

    // Return conf and keypairs
    (conf, keypairs)
}

/// Load configuration. at specified path
pub fn load_conf_at_path(profile: &str, profile_path: &PathBuf) -> (DuRsConf, DuniterKeyPairs) {
    // Get KeyPairs
    let mut keypairs_path = profile_path.clone();
    keypairs_path.push("keypairs.json");
    let keypairs = if keypairs_path.as_path().exists() {
        if let Ok(mut f) = File::open(keypairs_path.as_path()) {
            let mut contents = String::new();
            if f.read_to_string(&mut contents).is_ok() {
                let json_conf: serde_json::Value =
                    serde_json::from_str(&contents).expect("Conf: Fail to parse keypairs file !");
                if let Some(network_sec) = json_conf.get("network_sec") {
                    if let Some(network_pub) = json_conf.get("network_pub") {
                        let network_sec = network_sec
                            .as_str()
                            .expect("Conf: Fail to parse keypairs file !");
                        let network_pub = network_pub
                            .as_str()
                            .expect("Conf: Fail to parse keypairs file !");
                        DuniterKeyPairs {
                            network_keypair: KeyPairEnum::Ed25519(ed25519::KeyPair {
                                privkey: ed25519::PrivateKey::from_base58(network_sec)
                                    .expect("conf : keypairs file : fail to parse network_sec !"),
                                pubkey: ed25519::PublicKey::from_base58(network_pub)
                                    .expect("conf : keypairs file : fail to parse network_pub !"),
                            }),
                            member_keypair: None,
                        }
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
        write_keypairs_file(&keypairs_path, &keypairs)
            .expect("Fatal error : fail to write default keypairs file !");
        keypairs
    };

    // Open conf file
    let mut conf = DuRsConf::default();
    let mut conf_path = profile_path.clone();
    conf_path.push("conf.json");
    if conf_path.as_path().exists() {
        if let Ok(mut f) = File::open(conf_path.as_path()) {
            let mut contents = String::new();
            if f.read_to_string(&mut contents).is_ok() {
                conf = serde_json::from_str(&contents).expect("Conf: Fail to parse conf file !");
            }
        } else {
            panic!("Fail to open conf file !");
        }
    } else {
        // Create conf file with default conf
        write_conf_file(profile, &conf).expect("Fatal error : fail to write default conf file !");
    }

    // Return conf and keypairs
    (conf, keypairs)
}

/// Save keypairs in profile folder
pub fn write_keypairs_file(
    file_path: &PathBuf,
    keypairs: &DuniterKeyPairs,
) -> Result<(), std::io::Error> {
    let mut f = try!(File::create(file_path.as_path()));
    try!(
        f.write_all(
            serde_json::to_string_pretty(keypairs)
                .expect("Fatal error : fail to write default keypairs file !")
                .as_bytes()
        )
    );
    try!(f.sync_all());
    Ok(())
}

/// Save configuration in profile folder
pub fn write_conf_file<DC: DuniterConf>(profile: &str, conf: &DC) -> Result<(), std::io::Error> {
    let mut conf_path = get_profile_path(profile);
    conf_path.push("conf.json");
    let mut f = try!(File::create(conf_path.as_path()));
    f.write_all(
        serde_json::to_string_pretty(conf)
            .expect("Fatal error : fail to write default conf file !")
            .as_bytes(),
    )?;
    f.sync_all()?;
    Ok(())
}

/// Returns the path to the database containing the blockchain
pub fn get_blockchain_db_path(profile: &str, currency: &Currency) -> PathBuf {
    let mut db_path = datas_path(profile, &currency);
    db_path.push("blockchain/");
    if !db_path.as_path().exists() {
        fs::create_dir(db_path.as_path()).expect("Impossible to create blockchain dir !");
    }
    db_path
}

/// Returns the path to the binary file containing the state of the web of trust
pub fn get_wot_path(profile: String, currency: &Currency) -> PathBuf {
    let mut wot_path = match env::home_dir() {
        Some(path) => path,
        None => panic!("Impossible to get your home dir!"),
    };
    wot_path.push(".config/");
    wot_path.push(USER_DATAS_FOLDER);
    wot_path.push(profile);
    wot_path.push(currency.to_string());
    wot_path.push("wot.bin");
    wot_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_conf_file() {
        let (conf, _keys) = load_conf_at_path("test", &PathBuf::from("./test/"));
        assert_eq!(
            conf.modules()
                .get("ws2p")
                .expect("Not found ws2p conf")
                .clone(),
            json!({
                "sync_peers": [{
                    "pubkey": "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx",
                    "ws2p_endpoints": ["WS2P c1c39a0a i3.ifee.fr 80 /ws2p"]
                },{
                    "pubkey": "BoZP6aqtErHjiKLosLrQxBafi4ATciyDZQ6XRQkNefqG",
                    "ws2p_endpoints": ["WS2P 15af24db g1.ifee.fr 80 /ws2p"]
                },{
                    "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef",
                    "ws2p_endpoints": ["WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p"]
                }]
            })
        );
    }
}

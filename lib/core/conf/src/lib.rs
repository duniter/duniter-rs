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

//! Durs configuration files properties module

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
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod constants;
pub mod keys;

use crate::constants::MODULES_DATAS_FOLDER;
use dup_crypto::keys::*;
use dup_currency_params::CurrencyName;
use durs_common_tools::fatal_error;
use durs_module::{
    DursConfTrait, DursGlobalConfTrait, ModuleName, RequiredKeys, RequiredKeysContent,
};
use failure::Fail;
use rand::Rng;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
/// User request on global conf
pub enum ChangeGlobalConf {
    /// Change currency
    ChangeCurrency(CurrencyName),
    /// Disable module
    DisableModule(ModuleName),
    /// Enable module
    EnableModule(ModuleName),
    /// None
    None(),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Modules conf
pub struct ModulesConf(pub serde_json::Value);

impl Default for ModulesConf {
    #[inline]
    fn default() -> Self {
        ModulesConf(serde_json::Value::Null)
    }
}

impl ModulesConf {
    /// Change module conf
    pub fn set_module_conf(&mut self, module_name: ModuleName, new_module_conf: serde_json::Value) {
        if self.0.is_null() {
            let mut new_modules_conf = serde_json::Map::with_capacity(1);
            new_modules_conf.insert(module_name.0, new_module_conf);
            self.0 = serde_json::value::to_value(new_modules_conf)
                .expect("Fail to create map of new modules conf !");
        } else {
            self.0
                .as_object_mut()
                .expect("Conf file currupted !")
                .insert(module_name.0, new_module_conf);
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Duniter configuration v1
pub struct DuRsConfV1 {
    /// Currency name
    pub currency: CurrencyName,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Configuration of modules in json format (obtained from the conf.json file)
    pub modules: ModulesConf,
    /// Disabled modules
    pub disabled: HashSet<ModuleName>,
    /// Enabled modules
    pub enabled: HashSet<ModuleName>,
}

impl Default for DuRsConfV1 {
    fn default() -> Self {
        DuRsConfV1 {
            currency: CurrencyName(String::from(constants::DEFAULT_CURRENCY)),
            my_node_id: generate_random_node_id(),
            modules: ModulesConf::default(),
            disabled: HashSet::with_capacity(0),
            enabled: HashSet::with_capacity(0),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
/// Ressource usage
pub enum ResourceUsage {
    /// Minimal use of the resource, to the detriment of performance
    Minimal,
    /// Trade-off between resource use and performance
    Medium,
    /// A performance-oriented trade-off, the use of the resource is slightly limited
    Large,
    /// No restrictions on the use of the resource, maximizes performance
    Infinite,
}
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
/// Ressources usage
pub struct ResourcesUsage {
    /// Cpu usage
    pub cpu_usage: ResourceUsage,
    /// Network usage
    pub network_usage: ResourceUsage,
    /// Memory usage
    pub memory_usage: ResourceUsage,
    /// Disk space usage
    pub disk_space_usage: ResourceUsage,
}

impl Default for ResourcesUsage {
    fn default() -> Self {
        ResourcesUsage {
            cpu_usage: ResourceUsage::Large,
            network_usage: ResourceUsage::Large,
            memory_usage: ResourceUsage::Large,
            disk_space_usage: ResourceUsage::Large,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Duniter configuration v2
pub struct DuRsConfV2 {
    /// Currency name
    pub currency: CurrencyName,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Name of the module used by default for synchronization
    pub default_sync_module: ModuleName,
    /// Ressources usage
    pub ressources_usage: ResourcesUsage,
    /// Disabled modules
    pub disabled: HashSet<ModuleName>,
    /// Enabled modules
    pub enabled: HashSet<ModuleName>,
}

impl Default for DuRsConfV2 {
    fn default() -> Self {
        DuRsConfV2 {
            currency: CurrencyName(String::from(constants::DEFAULT_CURRENCY)),
            my_node_id: generate_random_node_id(),
            default_sync_module: ModuleName(String::from(constants::DEFAULT_DEFAULT_SYNC_MODULE)),
            ressources_usage: ResourcesUsage::default(),
            disabled: HashSet::with_capacity(0),
            enabled: HashSet::with_capacity(0),
        }
    }
}

impl From<DuRsConfV1> for DuRsConfV2 {
    fn from(conf_v1: DuRsConfV1) -> Self {
        DuRsConfV2 {
            currency: conf_v1.currency,
            my_node_id: conf_v1.my_node_id,
            default_sync_module: ModuleName(String::from(constants::DEFAULT_DEFAULT_SYNC_MODULE)),
            ressources_usage: ResourcesUsage::default(),
            disabled: conf_v1.disabled,
            enabled: conf_v1.enabled,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Durs node configuration
pub enum DuRsConf {
    /// Durs node configuration v1
    V1(DuRsConfV1),
    /// Durs node configuration v2
    V2 {
        /// Global configuration
        global_conf: DuRsConfV2,
        /// Modules configuration
        modules_conf: ModulesConf,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Durs global configuration (without modules configuration)
pub enum DuRsGlobalConf {
    /// Durs global configuration v1
    V1(DuRsConfV1),
    /// Durs global configuration v2
    V2(DuRsConfV2),
}

impl DursGlobalConfTrait for DuRsGlobalConf {
    fn my_node_id(&self) -> u32 {
        match *self {
            DuRsGlobalConf::V1(ref conf_v1) => conf_v1.my_node_id,
            DuRsGlobalConf::V2(ref conf_v2) => conf_v2.my_node_id,
        }
    }
    fn default_sync_module(&self) -> ModuleName {
        match *self {
            DuRsGlobalConf::V1(_) => {
                fatal_error!("Feature default_sync_module not exist in durs conf v1 !")
            }
            DuRsGlobalConf::V2(ref conf_v2) => conf_v2.default_sync_module.clone(),
        }
    }
}

impl Default for DuRsConf {
    #[inline]
    fn default() -> Self {
        DuRsConf::V2 {
            global_conf: DuRsConfV2::default(),
            modules_conf: ModulesConf::default(),
        }
    }
}

impl DursConfTrait for DuRsConf {
    type GlobalConf = DuRsGlobalConf;

    fn get_global_conf(&self) -> Self::GlobalConf {
        match *self {
            DuRsConf::V1(ref conf_v1) => DuRsGlobalConf::V1(conf_v1.clone()),
            DuRsConf::V2 {
                ref global_conf, ..
            } => DuRsGlobalConf::V2(global_conf.clone()),
        }
    }
    fn upgrade(self) -> (Self, bool) {
        if let DuRsConf::V1(conf_v1) = self {
            let modules_conf = conf_v1.modules.clone();
            (
                DuRsConf::V2 {
                    global_conf: DuRsConfV2::from(conf_v1),
                    modules_conf,
                },
                true,
            )
        } else {
            (self, false)
        }
    }
    fn version(&self) -> usize {
        match *self {
            DuRsConf::V1(_) => 1,
            DuRsConf::V2 { .. } => 2,
        }
    }
    fn set_currency(&mut self, new_currency: CurrencyName) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => conf_v1.currency = new_currency,
            DuRsConf::V2 {
                ref mut global_conf,
                ..
            } => global_conf.currency = new_currency,
        }
    }
    fn disable(&mut self, module: ModuleName) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => {
                conf_v1.disabled.insert(module.clone());
                conf_v1.enabled.remove(&module);
            }
            DuRsConf::V2 {
                ref mut global_conf,
                ..
            } => {
                global_conf.disabled.insert(module.clone());
                global_conf.enabled.remove(&module);
            }
        }
    }
    fn enable(&mut self, module: ModuleName) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => {
                conf_v1.disabled.remove(&module);
                conf_v1.enabled.insert(module);
            }
            DuRsConf::V2 {
                ref mut global_conf,
                ..
            } => {
                global_conf.disabled.remove(&module);
                global_conf.enabled.insert(module);
            }
        }
    }
    fn disabled_modules(&self) -> HashSet<ModuleName> {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.disabled.clone(),
            DuRsConf::V2 {
                ref global_conf, ..
            } => global_conf.disabled.clone(),
        }
    }
    fn enabled_modules(&self) -> HashSet<ModuleName> {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.enabled.clone(),
            DuRsConf::V2 {
                ref global_conf, ..
            } => global_conf.enabled.clone(),
        }
    }
    fn modules(&self) -> serde_json::Value {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.modules.0.clone(),
            DuRsConf::V2 {
                ref modules_conf, ..
            } => modules_conf.0.clone(),
        }
    }
    fn set_module_conf(&mut self, module_name: ModuleName, new_module_conf: serde_json::Value) {
        match *self {
            DuRsConf::V1(ref mut conf_v1) => conf_v1
                .modules
                .set_module_conf(module_name, new_module_conf),
            DuRsConf::V2 {
                ref mut modules_conf,
                ..
            } => modules_conf.set_module_conf(module_name, new_module_conf),
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

// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
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
    constants::USER_DATAS_FOLDER
}

/// Returns the path to the folder containing the modules datas of the running profile
#[inline]
pub fn get_datas_path(profile_path: PathBuf) -> PathBuf {
    let mut datas_path = profile_path;
    datas_path.push(MODULES_DATAS_FOLDER);
    if !datas_path.as_path().exists() {
        if let Err(io_error) = fs::create_dir(datas_path.as_path()) {
            if io_error.kind() != std::io::ErrorKind::AlreadyExists {
                fatal_error!("Impossible to create currency dir !");
            }
        }
    }
    datas_path
}

#[inline]
/// Return path to configuration file
pub fn get_conf_path(profile_path: &PathBuf) -> PathBuf {
    let mut conf_path = profile_path.clone();
    conf_path.push(constants::CONF_FILENAME);
    conf_path
}

/// Returns the path to the folder containing the user data of the running profile
// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
pub fn get_profile_path(profiles_path: &Option<PathBuf>, profile_name: &str) -> PathBuf {
    // Define and create datas directory if not exist
    let profiles_path: PathBuf = if let Some(profiles_path) = profiles_path {
        profiles_path.clone()
    } else {
        let mut user_config_path = match dirs::config_dir() {
            Some(path) => path,
            None => panic!("Impossible to get user config directory !"),
        };
        user_config_path.push(constants::USER_DATAS_FOLDER);
        user_config_path
    };
    if !profiles_path.as_path().exists() {
        fs::create_dir(profiles_path.as_path()).unwrap_or_else(|_| {
            panic!(
                "Impossible to create profiles directory: {:?} !",
                profiles_path
            )
        });
    }
    let mut profile_path = profiles_path;
    profile_path.push(profile_name);
    if !profile_path.as_path().exists() {
        fs::create_dir(profile_path.as_path()).expect("Impossible to create your profile dir !");
    }
    profile_path
}

/// Get keypairs file path
pub fn keypairs_filepath(profiles_path: &Option<PathBuf>, profile: &str) -> PathBuf {
    let profile_path = get_profile_path(profiles_path, profile);
    let mut conf_keys_path = profile_path.clone();
    conf_keys_path.push(constants::KEYPAIRS_FILENAME);
    conf_keys_path
}

/// Load configuration.
pub fn load_conf(
    profile_path: PathBuf,
    keypairs_file_path: &Option<PathBuf>,
) -> Result<(DuRsConf, DuniterKeyPairs), DursConfFileError> {
    // Load conf
    let (conf, keypairs) = load_conf_at_path(profile_path.clone(), keypairs_file_path)?;

    // Return conf and keypairs
    Ok((conf, keypairs))
}

/// Error with configuration file
#[derive(Debug, Fail)]
pub enum DursConfFileError {
    /// Read error
    #[fail(display = "fail to read configuration file: {}", _0)]
    ReadError(std::io::Error),
    /// Parse error
    #[fail(display = "fail to parse configuration file: {}", _0)]
    ParseError(serde_json::Error),
    /// Write error
    #[fail(display = "fail to write configuration file: {}", _0)]
    WriteError(std::io::Error),
}

/// Load configuration. at specified path
// Warning: This function cannot use the macro fatal_error! because the logger is not yet initialized, so it must use panic !
pub fn load_conf_at_path(
    profile_path: PathBuf,
    keypairs_file_path: &Option<PathBuf>,
) -> Result<(DuRsConf, DuniterKeyPairs), DursConfFileError> {
    // Get KeyPairs
    let keypairs_path = if let Some(ref keypairs_file_path) = keypairs_file_path {
        keypairs_file_path.clone()
    } else {
        let mut keypairs_path = profile_path.clone();
        keypairs_path.push(constants::KEYPAIRS_FILENAME);
        keypairs_path
    };
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
                        let network_keypair = KeyPairEnum::Ed25519(ed25519::KeyPair {
                            privkey: ed25519::PrivateKey::from_base58(network_sec)
                                .expect("conf : keypairs file : fail to parse network_sec !"),
                            pubkey: ed25519::PublicKey::from_base58(network_pub)
                                .expect("conf : keypairs file : fail to parse network_pub !"),
                        });

                        let member_keypair = if let Some(member_sec) = json_conf.get("member_sec") {
                            if let Some(member_pub) = json_conf.get("member_pub") {
                                let member_sec = member_sec
                                    .as_str()
                                    .expect("Conf: Fail to parse keypairs file !");
                                let member_pub = member_pub
                                    .as_str()
                                    .expect("Conf: Fail to parse keypairs file !");
                                if member_sec.is_empty() || member_pub.is_empty() {
                                    None
                                } else {
                                    Some(KeyPairEnum::Ed25519(ed25519::KeyPair {
                                        privkey: ed25519::PrivateKey::from_base58(member_sec)
                                            .expect(
                                                "conf : keypairs file : fail to parse member_sec !",
                                            ),
                                        pubkey: ed25519::PublicKey::from_base58(member_pub).expect(
                                            "conf : keypairs file : fail to parse member_pub !",
                                        ),
                                    }))
                                }
                            } else {
                                panic!("Fatal error : keypairs file wrong format : no field salt !")
                            }
                        } else {
                            panic!("Fatal error : keypairs file wrong format : no field password !")
                        };

                        // Create keypairs file with random keypair
                        DuniterKeyPairs {
                            network_keypair,
                            member_keypair,
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
        write_keypairs_file(&keypairs_path, &keypairs).unwrap_or_else(|_| {
            panic!(dbg!("Fatal error : fail to write default keypairs file !"))
        });
        keypairs
    };

    // Open conf file
    let mut conf_path = profile_path;
    conf_path.push(constants::CONF_FILENAME);
    let conf = if conf_path.as_path().exists() {
        match File::open(conf_path.as_path()) {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents)
                    .map_err(DursConfFileError::ReadError)?;
                // Parse conf file
                let conf: DuRsConf =
                    serde_json::from_str(&contents).map_err(DursConfFileError::ParseError)?;
                // Upgrade conf to latest version
                let (conf, upgraded) = conf.upgrade();
                // If conf is upgraded, rewrite conf file
                if upgraded {
                    write_conf_file(conf_path.as_path(), &conf)
                        .map_err(DursConfFileError::WriteError)?;
                }
                conf
            }
            Err(e) => return Err(DursConfFileError::ReadError(e)),
        }
    } else {
        // Create conf file with default conf
        let conf = DuRsConf::default();
        write_conf_file(conf_path.as_path(), &conf)
            .unwrap_or_else(|_| panic!(dbg!("Fatal error : fail to write default conf file!")));
        conf
    };

    // Return conf and keypairs
    Ok((conf, keypairs))
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

/// Write new module conf
pub fn write_new_module_conf<DC: DursConfTrait>(
    conf: &mut DC,
    profile_path: PathBuf,
    module_name: ModuleName,
    new_module_conf: serde_json::Value,
) {
    conf.set_module_conf(module_name, new_module_conf);
    let mut conf_path = profile_path;
    conf_path.push(crate::constants::CONF_FILENAME);
    write_conf_file(conf_path.as_path(), conf).expect("Fail to write new conf file ! ");
}

/// Save configuration in profile folder
pub fn write_conf_file<DC: DursConfTrait>(
    conf_path: &Path,
    conf: &DC,
) -> Result<(), std::io::Error> {
    let mut f = File::create(conf_path)?;
    f.write_all(
        serde_json::to_string_pretty(conf)
            .expect("Fatal error : fail to write default conf file !")
            .as_bytes(),
    )?;
    f.sync_all()?;
    Ok(())
}

/// Returns the path to the database containing the blockchain
pub fn get_blockchain_db_path(profile_path: PathBuf) -> PathBuf {
    let mut db_path = get_datas_path(profile_path);
    db_path.push("blockchain/");
    if !db_path.as_path().exists() {
        if let Err(io_error) = fs::create_dir(db_path.as_path()) {
            if io_error.kind() != std::io::ErrorKind::AlreadyExists {
                fatal_error!("Impossible to create blockchain dir !");
            }
        }
    }
    db_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[inline]
    fn save_old_conf(profile_path: PathBuf) -> std::io::Result<()> {
        let mut conf_path = profile_path.clone();
        conf_path.push(constants::CONF_FILENAME);
        let mut conf_sav_path = profile_path;
        conf_sav_path.push("conf-sav.json");
        std::fs::copy(conf_path.as_path(), conf_sav_path.as_path())?;
        Ok(())
    }

    fn restore_old_conf_and_save_upgraded_conf(profile_path: PathBuf) -> std::io::Result<()> {
        let mut conf_path = profile_path.clone();
        conf_path.push(constants::CONF_FILENAME);
        let mut conf_sav_path = profile_path.clone();
        conf_sav_path.push("conf-sav.json");
        let mut conf_upgraded_path = profile_path;
        conf_upgraded_path.push("conf-upgraded.json");
        std::fs::copy(conf_path.as_path(), &conf_upgraded_path.as_path())?;
        std::fs::copy(conf_sav_path.as_path(), &conf_path.as_path())?;
        std::fs::remove_file(conf_sav_path.as_path())?;
        Ok(())
    }

    #[test]
    fn load_conf_file_v1() -> Result<(), DursConfFileError> {
        let profile_path = PathBuf::from("./test/v1/");
        save_old_conf(PathBuf::from(profile_path.clone()))
            .map_err(DursConfFileError::WriteError)?;
        let (conf, _keys) = load_conf_at_path(profile_path.clone(), &None)?;
        assert_eq!(
            conf.modules()
                .get("ws2p")
                .expect("Not found ws2p conf")
                .clone(),
            json!({
                "sync_endpoints": [
                {
                    "endpoint": "WS2P c1c39a0a i3.ifee.fr 80 /ws2p",
                    "pubkey": "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx"
                },
                {
                    "endpoint": "WS2P 15af24db g1.ifee.fr 80 /ws2p",
                    "pubkey": "BoZP6aqtErHjiKLosLrQxBafi4ATciyDZQ6XRQkNefqG"
                },
                {
                    "endpoint": "WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p",
                    "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef"
                }
                ]
            })
        );
        restore_old_conf_and_save_upgraded_conf(profile_path)
            .map_err(DursConfFileError::WriteError)?;

        Ok(())
    }

    #[test]
    fn load_conf_file_v2() -> Result<(), DursConfFileError> {
        let profile_path = PathBuf::from("./test/v2/");
        let (conf, _keys) = load_conf_at_path(profile_path, &None)?;
        assert_eq!(
            conf.modules()
                .get("ws2p")
                .expect("Not found ws2p conf")
                .clone(),
            json!({
                "sync_endpoints": [
                {
                    "endpoint": "WS2P c1c39a0a i3.ifee.fr 80 /ws2p",
                    "pubkey": "D9D2zaJoWYWveii1JRYLVK3J4Z7ZH3QczoKrnQeiM6mx"
                },
                {
                    "endpoint": "WS2P 15af24db g1.ifee.fr 80 /ws2p",
                    "pubkey": "BoZP6aqtErHjiKLosLrQxBafi4ATciyDZQ6XRQkNefqG"
                },
                {
                    "endpoint": "WS2P b48824f0 g1.monnaielibreoccitanie.org 80 /ws2p",
                    "pubkey": "7v2J4badvfWQ6qwRdCwhhJfAsmKwoxRUNpJHiJHj7zef"
                }
                ]
            })
        );
        Ok(())
    }
}

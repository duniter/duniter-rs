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

//! Dunitrust configuration module

#![deny(
    clippy::option_unwrap_used,
    clippy::result_unwrap_used,
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
mod env;
pub mod errors;
pub mod file;
mod global_conf;
pub mod keypairs;
mod modules_conf;
mod resources;
mod v1;

pub use crate::errors::DursConfError;
pub use crate::keypairs::DuniterKeyPairs;

use crate::constants::MODULES_DATAS_FOLDER;
use crate::global_conf::v2::DuRsGlobalConfV2;
use crate::global_conf::{DuRsGlobalConf, DuRsGlobalUserConf};
use crate::modules_conf::ModulesConf;
use dubp_currency_params::CurrencyName;
use dup_crypto::keys::*;
use dup_crypto::rand;
use durs_common_tools::fatal_error;
use durs_module::{DursConfTrait, DursGlobalConfTrait, ModuleName};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Dunitrust node configuration
pub enum DuRsConf {
    /// Dunitrust node configuration v1
    V1(v1::DuRsConfV1),
    /// Dunitrust node configuration v2
    V2 {
        /// Global configuration
        global_conf: DuRsGlobalConfV2,
        /// Modules configuration
        modules_conf: ModulesConf,
    },
}

impl Default for DuRsConf {
    #[inline]
    fn default() -> Self {
        DuRsConf::V2 {
            global_conf: DuRsGlobalConfV2::default(),
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
    fn override_global_conf(
        self,
        global_user_conf: <Self::GlobalConf as DursGlobalConfTrait>::GlobalUserConf,
    ) -> Self {
        match self {
            DuRsConf::V1(conf_v1) => DuRsConf::V1(conf_v1),
            DuRsConf::V2 {
                global_conf,
                modules_conf,
            } => {
                let DuRsGlobalUserConf::V2(global_user_conf_v2) = global_user_conf;
                DuRsConf::V2 {
                    global_conf: global_conf.r#override(global_user_conf_v2),
                    modules_conf,
                }
            }
        }
    }
    fn upgrade(self) -> (Self, bool) {
        if let DuRsConf::V1(conf_v1) = self {
            let modules_conf = conf_v1.modules.clone();
            (
                DuRsConf::V2 {
                    global_conf: DuRsGlobalConfV2::from(conf_v1),
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
    fn get_currency(&self) -> CurrencyName {
        match *self {
            DuRsConf::V1(ref conf_v1) => conf_v1.currency.clone(),
            DuRsConf::V2 {
                ref global_conf, ..
            } => global_conf.currency.clone(),
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

#[inline]
fn generate_random_node_id() -> u32 {
    rand::gen_u32()
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
                fatal_error!("Impossible to create modules datas folder !");
            }
        }
    }
    datas_path
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

/// Load configuration.
pub fn load_conf(
    profile_path: PathBuf,
    keypairs_file_path: &Option<PathBuf>,
) -> Result<(DuRsConf, DuniterKeyPairs), DursConfError> {
    let keypairs = crate::keypairs::load_keypairs_from_file(&profile_path, keypairs_file_path)?;

    // Load conf from file
    let conf_from_file =
        crate::file::load_conf_from_file(profile_path).map_err(DursConfError::FileErr)?;

    // Try to load global user conf from env vars
    let env_global_user_conf =
        env::load_env_global_user_conf().map_err(DursConfError::EnvVarErr)?;

    // Override global conf with env global user conf
    let conf = conf_from_file.override_global_conf(env_global_user_conf);

    Ok((conf, keypairs))
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
    crate::file::write_conf_file(conf_path.as_path(), conf)
        .expect("Fail to write new conf file ! ");
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

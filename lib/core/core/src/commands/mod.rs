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

//! Define durs-core cli subcommands options.

pub mod dbex;
pub mod keys;
pub mod modules;
pub mod reset;
pub mod start;

use crate::constants::DEFAULT_USER_PROFILE;
use crate::errors::DursCoreError;
use crate::DursCore;
pub use dbex::*;
use durs_conf::DuRsConf;
use durs_dbs_tools::kv_db_old::KvFileDbHandler;
pub use durs_network::cli::sync::SyncOpt;
pub use keys::KeysOpt;
use log::Level;
pub use modules::*;
pub use reset::*;
pub use start::*;
use std::path::PathBuf;

/// Dunitrust core options
pub struct DursCoreOptions {
    /// Keypairs file path
    pub keypairs_file: Option<PathBuf>,
    /// Set log level.
    pub logs_level: Level,
    /// Print logs in standard output
    pub log_stdout: bool,
    /// Set a custom user profile name
    pub profile_name: Option<String>,
    /// Path where user profiles are persisted
    pub profiles_path: Option<PathBuf>,
}

impl DursCoreOptions {
    /// Define profile path
    #[inline]
    pub fn define_profile_path(&self) -> PathBuf {
        durs_conf::get_profile_path(
            &self.profiles_path,
            &self
                .profile_name
                .clone()
                .unwrap_or_else(|| DEFAULT_USER_PROFILE.to_owned()),
        )
    }
}

/// Dunitrust executable command
pub trait DursExecutableCoreCommand {
    /// Execute Dunitrust command
    fn execute(self, durs_core: DursCore<DuRsConf>) -> Result<(), DursCoreError>;
}

/// Executable module command
pub trait ExecutableModuleCommand {
    /// Execute module command
    fn execute_module_command(self, options: DursCoreOptions) -> Result<(), DursCoreError>;
}

/// Dunitrust command with options
pub struct DursCommand<T: ExecutableModuleCommand> {
    /// Dunitrust core options
    pub options: DursCoreOptions,
    /// Dunitrust command
    pub command: DursCommandEnum<T>,
}

/// Dunitrust command
pub enum DursCommandEnum<T: ExecutableModuleCommand> {
    /// Core command
    Core(DursCoreCommand),
    /// Other command
    Other(T),
}

impl<T: ExecutableModuleCommand> DursCommand<T> {
    fn open_bc_db(&self, profile_path: &PathBuf) -> Result<KvFileDbHandler, DursCoreError> {
        let bc_db_path = durs_conf::get_blockchain_db_path(profile_path.clone());
        durs_dbs_tools::kv_db_old::KvFileDbHandler::open_db(
            bc_db_path.as_path(),
            &durs_bc_db_reader::bc_db_schema(),
        )
        .map_err(DursCoreError::FailOpenBcDb)
    }
    /// Execute Dunitrust command
    pub fn execute<PlugFunc>(
        self,
        soft_name: &'static str,
        soft_version: &'static str,
        plug_modules: PlugFunc,
    ) -> Result<(), DursCoreError>
    where
        PlugFunc: FnMut(&mut DursCore<DuRsConf>) -> Result<(), DursCoreError>,
    {
        let profile_path = self.options.define_profile_path();
        let bc_db = self.open_bc_db(&profile_path)?;

        match self.command {
            DursCommandEnum::Core(core_cmd) => DursCore::execute_core_command(
                bc_db,
                core_cmd,
                self.options,
                plug_modules,
                profile_path,
                soft_name,
                soft_version,
            ),
            DursCommandEnum::Other(cmd) => cmd.execute_module_command(self.options),
        }
    }
}

#[derive(StructOpt, Debug)]
/// Core cli subcommands
pub enum DursCoreCommand {
    /// Enable a module
    EnableOpt(EnableOpt),
    /// Disable a module
    DisableOpt(DisableOpt),
    /// List available modules
    ListModulesOpt(ListModulesOpt),
    /// Start node
    StartOpt(StartOpt),
    /// Synchronize
    SyncOpt(SyncOpt),
    /// Reset data or conf or all
    ResetOpt(ResetOpt),
    /// Database explorer
    DbExOpt(DbExOpt),
    /// Keys operations
    KeysOpt(KeysOpt),
}

/// InvalidInput
#[derive(Debug, Copy, Clone)]
pub struct InvalidInput(&'static str);

impl ToString for InvalidInput {
    fn to_string(&self) -> String {
        String::from(self.0)
    }
}

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

//! Command line options for classic Dunitrust nodes (no specialization).

use durs_core::commands::dbex::DbExOpt;
use durs_core::commands::keys::KeysOpt;
use durs_core::commands::modules::{DisableOpt, EnableOpt, ListModulesOpt};
use durs_core::commands::reset::ResetOpt;
use durs_core::commands::start::StartOpt;
use durs_core::commands::{
    DursCommand, DursCommandEnum, DursCoreCommand, DursCoreOptions, ExecutableModuleCommand,
};
use durs_core::errors::DursCoreError;
use durs_core::DursCore;
use durs_gva::{GvaModule, GvaOpt};
use durs_network::cli::sync::SyncOpt;
use durs_ws2p_v1_legacy::{WS2POpt, WS2Pv1Module};
use log::Level;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "durs", setting(structopt::clap::AppSettings::ColoredHelp))]
/// Dunitrust command line options
pub struct DursCliOpt {
    /// Dunitrust subcommand
    #[structopt(subcommand)]
    cmd: DursCliSubCommand,
    /// Path where user profiles are persisted
    #[structopt(long = "profiles-path", parse(from_os_str))]
    profiles_path: Option<PathBuf>,
    /// Keypairs file path
    #[structopt(long = "keypairs-file", parse(from_os_str))]
    keypairs_file: Option<PathBuf>,
    /// Set log level. (Defaults to INFO).
    /// Available levels: [ERROR, WARN, INFO, DEBUG, TRACE]
    #[structopt(short = "l", long = "logs", next_line_help = true)]
    logs_level: Option<Level>,
    /// Print logs in standard output
    #[structopt(long = "log-stdout")]
    log_stdout: bool,
    /// Set a custom user profile name
    #[structopt(short = "p", long = "profile-name")]
    profile_name: Option<String>,
}

impl ExecutableModuleCommand for DursCliOpt {
    /// Execute command
    fn execute_module_command(self, options: DursCoreOptions) -> Result<(), DursCoreError> {
        match self.cmd {
            DursCliSubCommand::Ws2p1(module_opts) => {
                DursCore::execute_module_command::<WS2Pv1Module>(
                    options,
                    module_opts,
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                )
            }
            DursCliSubCommand::Gva(module_opts) => DursCore::execute_module_command::<GvaModule>(
                options,
                module_opts,
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ),
            _ => unreachable!(),
        }
    }
}

impl DursCliOpt {
    /// Into Dunitrust command
    pub fn into_durs_command(self) -> DursCommand<DursCliOpt> {
        let options = DursCoreOptions {
            keypairs_file: self.keypairs_file.clone(),
            logs_level: self.logs_level,
            log_stdout: self.log_stdout,
            profile_name: self.profile_name.clone(),
            profiles_path: self.profiles_path.clone(),
        };

        match self.cmd {
            DursCliSubCommand::DbExOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::DbExOpt(opts)),
            },
            DursCliSubCommand::DisableOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::DisableOpt(opts)),
            },
            DursCliSubCommand::EnableOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::EnableOpt(opts)),
            },
            DursCliSubCommand::KeysOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::KeysOpt(opts)),
            },
            DursCliSubCommand::ListModulesOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::ListModulesOpt(opts)),
            },
            DursCliSubCommand::ResetOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::ResetOpt(opts)),
            },
            DursCliSubCommand::StartOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::StartOpt(opts)),
            },
            DursCliSubCommand::SyncOpt(opts) => DursCommand {
                options,
                command: DursCommandEnum::Core(DursCoreCommand::SyncOpt(opts)),
            },
            _ => DursCommand {
                options,
                command: DursCommandEnum::Other(self),
            },
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Classic Dunitrust nodes subcommand
pub enum DursCliSubCommand {
    /// Database explorer
    #[structopt(name = "dbex", setting(structopt::clap::AppSettings::ColoredHelp))]
    DbExOpt(DbExOpt),
    /// Disable a module
    #[structopt(name = "disable", setting(structopt::clap::AppSettings::ColoredHelp))]
    DisableOpt(DisableOpt),
    /// Enable a module
    #[structopt(name = "enable", setting(structopt::clap::AppSettings::ColoredHelp))]
    EnableOpt(EnableOpt),
    /// Keys operations
    #[structopt(
        name = "keys",
        author = "inso <inso@tuta.io>",
        setting(structopt::clap::AppSettings::ColoredHelp)
    )]
    KeysOpt(KeysOpt),
    /// List available modules
    #[structopt(name = "modules", setting(structopt::clap::AppSettings::ColoredHelp))]
    ListModulesOpt(ListModulesOpt),
    /// Reset data or conf or all
    #[structopt(name = "reset", setting(structopt::clap::AppSettings::ColoredHelp))]
    ResetOpt(ResetOpt),
    /// Start node
    #[structopt(name = "start", setting(structopt::clap::AppSettings::ColoredHelp))]
    StartOpt(StartOpt),
    /// Synchronize
    #[structopt(name = "sync", setting(structopt::clap::AppSettings::ColoredHelp))]
    SyncOpt(SyncOpt),
    /// GVA module subcommand
    #[structopt(name = "gva", setting(structopt::clap::AppSettings::ColoredHelp))]
    Gva(GvaOpt),
    /// WS2P1 module subcommand
    #[structopt(name = "ws2p1", setting(structopt::clap::AppSettings::ColoredHelp))]
    Ws2p1(WS2POpt),
}

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

//! Define durs-core cli subcommands options.

pub mod dbex;
pub mod keys;
pub mod modules;
pub mod reset;
pub mod start;

pub use crate::cli::keys::KeysOpt;
pub use crate::dbex::*;
pub use crate::modules::*;
pub use crate::reset::*;
pub use crate::start::*;
pub use duniter_network::cli::sync::SyncOpt;
use log::Level;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "durs",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Durs command line options
pub struct DursOpt {
    /// CoreSubCommand
    #[structopt(subcommand)]
    cmd: CoreSubCommand,
    /// Path where user profiles are persisted
    #[structopt(long = "profiles-path", parse(from_os_str))]
    profiles_path: Option<PathBuf>,
    /// Keypairs file path
    #[structopt(long = "keypairs-file", parse(from_os_str))]
    keypairs_file: Option<PathBuf>,
    /// Set log level. (Defaults to INFO).
    /// Available levels: [ERROR, WARN, INFO, DEBUG, TRACE]
    #[structopt(short = "l", long = "logs", raw(next_line_help = "true"))]
    logs_level: Option<Level>,
    /// Print logs in standard output
    #[structopt(long = "log-stdout")]
    log_stdout: bool,
    /// Set a custom user profile name
    #[structopt(short = "p", long = "profile-name")]
    profile_name: Option<String>,
}

#[derive(StructOpt, Debug)]
/// Core cli subcommands
pub enum CoreSubCommand {
    #[structopt(name = "enable")]
    /// Enable a module
    EnableOpt(EnableOpt),
    #[structopt(name = "disable")]
    /// Disable a module
    DisableOpt(DisableOpt),
    #[structopt(name = "modules")]
    /// List available modules
    ListModulesOpt(ListModulesOpt),
    #[structopt(name = "start")]
    /// Start node
    StartOpt(StartOpt),
    #[structopt(name = "sync")]
    /// Synchronize
    SyncOpt(SyncOpt),
    /// Reset data or conf or all
    #[structopt(
        name = "reset",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    ResetOpt(ResetOpt),
    /// Database explorer
    #[structopt(
        name = "dbex",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    DbExOpt(DbExOpt),
    /// Keys operations
    #[structopt(
        name = "keys",
        author = "inso <inso@tuta.io>",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
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

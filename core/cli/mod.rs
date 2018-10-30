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

extern crate structopt;

pub mod dbex;
pub mod keys;
pub mod modules;
pub mod reset;
pub mod start;
pub mod sync;

use cli::keys::KeysOpt;
pub use dbex::*;
pub use keys::*;
use log::Level;
pub use modules::*;
pub use reset::*;
pub use start::*;
pub use sync::*;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "durs",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Rust implementation of Duniter
pub struct DursOpt {
    #[structopt(short = "p", long = "profile")]
    /// Set a custom user datas folder
    profile_name: Option<String>,
    #[structopt(short = "l", long = "logs", raw(next_line_help = "true"))]
    /// Set the level of logs verbosity. (Default is INFO).
    /// Possible values : [OFF, ERROR, WARN, INFO, DEBUG, TRACE]
    logs_level: Option<Level>,
    #[structopt(subcommand)]
    /// CoreSubCommand
    cmd: CoreSubCommand,
}

#[derive(StructOpt, Debug)]
/// Core cli subcommands
pub enum CoreSubCommand {
    #[structopt(name = "enable")]
    /// Enable some module
    EnableOpt(EnableOpt),
    #[structopt(name = "disable")]
    /// Disable some module
    DisableOpt(DisableOpt),
    #[structopt(name = "modules")]
    /// list modules
    ListModulesOpt(ListModulesOpt),
    #[structopt(name = "start")]
    /// start durs server
    StartOpt(StartOpt),
    #[structopt(name = "sync")]
    /// synchronization from network
    SyncOpt(SyncOpt),
    #[structopt(name = "sync_ts")]
    /// synchronization via a duniter-ts database
    SyncTsOpt(SyncTsOpt),
    /// reset data or conf or all
    #[structopt(
        name = "reset",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    ResetOpt(ResetOpt),
    /// durs databases explorer
    #[structopt(
        name = "dbex",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    DbExOpt(DbExOpt),
    /// keys operations
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

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

//! Durs-core cli : keys subcommands.

use crate::commands::DursExecutableCoreCommand;
use crate::errors::DursCoreError;
use crate::DursCore;
use durs_conf::keys::*;
use durs_conf::DuRsConf;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "keys",
    author = "inso <inso@tuta.io>",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// keys management
pub struct KeysOpt {
    #[structopt(subcommand)]
    /// KeysSubCommand
    pub subcommand: KeysSubCommand,
}

#[derive(StructOpt, Debug, Clone)]
/// keys subcommands
pub enum KeysSubCommand {
    /// Modify keys
    #[structopt(
        name = "modify",
        author = "inso <inso@tuta.io>",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Modify(ModifyOpt),

    /// Clear keys
    #[structopt(
        name = "clear",
        author = "inso <inso@tuta.io>",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Clear(ClearOpt),

    /// Show keys
    #[structopt(
        name = "show",
        author = "inso <inso@tuta.io>",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Show(ShowOpt),

    #[structopt(
        name = "wizard",
        author = "inso <inso@tuta.io>",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// Keys generator wizard
    Wizard(WizardOpt),
}

#[derive(StructOpt, Debug, Clone)]
/// ModifyOpt
pub struct ModifyOpt {
    #[structopt(subcommand)]
    /// Modify sub commands
    pub subcommand: ModifySubCommand,
}

#[derive(StructOpt, Debug, Clone)]
/// keys modify subcommands
pub enum ModifySubCommand {
    #[structopt(
        name = "member",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// Salt and password of member key
    MemberSaltPassword(SaltPasswordOpt),

    #[structopt(
        name = "network",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    /// Salt and password of network key    
    NetworkSaltPassword(SaltPasswordOpt),
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// ClearOpt
pub struct ClearOpt {
    #[structopt(short = "m", long = "member")]
    /// True if we change member key
    pub member: bool,

    #[structopt(short = "n", long = "network")]
    /// True if we change network key
    pub network: bool,

    #[structopt(short = "a", long = "all")]
    /// True if we change member and network key
    pub all: bool,
}

#[derive(StructOpt, Debug, Clone)]
/// SaltPasswordOpt
pub struct SaltPasswordOpt {
    #[structopt(long = "salt")]
    /// Salt of key generator
    pub salt: String,

    #[structopt(long = "password")]
    /// Password of key generator
    pub password: String,
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// WizardOpt
pub struct WizardOpt {}

#[derive(StructOpt, Debug, Copy, Clone)]
/// ShowOpt
pub struct ShowOpt {}

impl DursExecutableCoreCommand for KeysOpt {
    fn execute(self, durs_core: DursCore<DuRsConf>) -> Result<(), DursCoreError> {
        let profile_path = durs_core.soft_meta_datas.profile_path;
        let keypairs_file = durs_core.options.keypairs_file;
        let keypairs = durs_core.keypairs;

        match self.subcommand {
            KeysSubCommand::Wizard(_) => {
                let new_keypairs = key_wizard(keypairs).unwrap();
                save_keypairs(profile_path, &keypairs_file, new_keypairs)
                    .map_err(DursCoreError::FailWriteKeypairsFile)
            }
            KeysSubCommand::Modify(modify_opt) => match modify_opt.subcommand {
                ModifySubCommand::NetworkSaltPassword(network_opt) => {
                    let new_keypairs =
                        modify_network_keys(&network_opt.salt, &network_opt.password, keypairs);
                    save_keypairs(profile_path, &keypairs_file, new_keypairs)
                        .map_err(DursCoreError::FailWriteKeypairsFile)
                }
                ModifySubCommand::MemberSaltPassword(member_opt) => {
                    let new_keypairs =
                        modify_member_keys(&member_opt.salt, &member_opt.password, keypairs);
                    save_keypairs(profile_path, &keypairs_file, new_keypairs)
                        .map_err(DursCoreError::FailWriteKeypairsFile)
                }
            },
            KeysSubCommand::Clear(clear_opt) => {
                let new_keypairs = clear_keys(
                    clear_opt.network || clear_opt.all,
                    clear_opt.member || clear_opt.all,
                    keypairs,
                );
                save_keypairs(profile_path, &keypairs_file, new_keypairs)
                    .map_err(DursCoreError::FailWriteKeypairsFile)
            }
            KeysSubCommand::Show(_) => {
                show_keys(keypairs);
                Ok(())
            }
        }
    }
}

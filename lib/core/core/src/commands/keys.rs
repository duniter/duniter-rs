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
use clap::arg_enum;
use durs_conf::keypairs::cli::*;
use durs_conf::DuRsConf;

#[derive(StructOpt, Debug, Clone, Copy)]
#[structopt(
    name = "keys",
    author = "inso <inso@tuta.io>",
    setting(structopt::clap::AppSettings::ColoredHelp)
)]
/// keys management
pub struct KeysOpt {
    #[structopt(subcommand)]
    /// KeysSubCommand
    pub subcommand: KeysSubCommand,
}

#[derive(StructOpt, Debug, Clone, Copy)]
/// keys subcommands
pub enum KeysSubCommand {
    /// Modify keys
    #[structopt(
        name = "modify",
        author = "inso <inso@tuta.io>",
        setting(structopt::clap::AppSettings::ColoredHelp)
    )]
    Modify(ModifyOpt),

    /// Clear keys
    #[structopt(
        name = "clear",
        author = "inso <inso@tuta.io>",
        setting(structopt::clap::AppSettings::ColoredHelp)
    )]
    Clear(ClearOpt),

    /// Show keys
    #[structopt(
        name = "show",
        author = "inso <inso@tuta.io>",
        setting(structopt::clap::AppSettings::ColoredHelp)
    )]
    Show(ShowOpt),

    #[structopt(
        name = "wizard",
        author = "inso <inso@tuta.io>",
        setting(structopt::clap::AppSettings::ColoredHelp)
    )]
    /// Keys generator wizard
    Wizard(WizardOpt),
}

#[derive(StructOpt, Debug, Clone, Copy)]
/// ModifyOpt
pub struct ModifyOpt {
    #[structopt(subcommand)]
    /// Modify sub commands
    pub subcommand: ModifySubCommand,
}

#[derive(StructOpt, Debug, Clone, Copy)]
/// keys modify subcommands
pub enum ModifySubCommand {
    #[structopt(name = "member", setting(structopt::clap::AppSettings::ColoredHelp))]
    /// Salt and password of member key
    MemberSaltPassword,

    #[structopt(name = "network", setting(structopt::clap::AppSettings::ColoredHelp))]
    /// Salt and password of network key    
    NetworkSaltPassword,
}

arg_enum! {
    /// KeyKind
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum KeyKind {
        MEMBER,
        NETWORK,
        ALL,
    }
}

impl KeyKind {
    /// Returns if key kind is member
    pub fn is_member(self) -> bool {
        self == KeyKind::MEMBER || self == KeyKind::ALL
    }
    /// Returns if key kind is network
    pub fn is_network(self) -> bool {
        self == KeyKind::NETWORK || self == KeyKind::ALL
    }
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// ClearOpt
pub struct ClearOpt {
    /// Key to clear
    #[structopt(possible_values = &KeyKind::variants(), case_insensitive = true)]
    key: KeyKind,
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
                let new_keypairs = key_wizard(keypairs)?;
                save_keypairs(profile_path, &keypairs_file, &new_keypairs)
                    .map_err(DursCoreError::FailWriteKeypairsFile)
                    .and_then(|_| {
                        show_keys(new_keypairs);
                        Ok(())
                    })
            }
            KeysSubCommand::Modify(modify_opt) => match modify_opt.subcommand {
                ModifySubCommand::NetworkSaltPassword => {
                    let new_keypairs = modify_network_keys(keypairs)?;
                    save_keypairs(profile_path, &keypairs_file, &new_keypairs)
                        .map_err(DursCoreError::FailWriteKeypairsFile)
                        .and_then(|_| {
                            show_network_keys(&new_keypairs);
                            Ok(())
                        })
                }
                ModifySubCommand::MemberSaltPassword => {
                    let new_keypairs = modify_member_keys(keypairs)?;
                    save_keypairs(profile_path, &keypairs_file, &new_keypairs)
                        .map_err(DursCoreError::FailWriteKeypairsFile)
                        .and_then(|_| {
                            show_member_keys(&new_keypairs);
                            Ok(())
                        })
                }
            },
            KeysSubCommand::Clear(clear_opt) => {
                let new_keypairs = clear_keys(
                    clear_opt.key.is_network(),
                    clear_opt.key.is_member(),
                    keypairs,
                );
                save_keypairs(profile_path, &keypairs_file, &new_keypairs)
                    .map_err(DursCoreError::FailWriteKeypairsFile)
                    .and_then(|_| {
                        show_keys(new_keypairs);
                        Ok(())
                    })
            }
            KeysSubCommand::Show(_) => {
                show_keys(keypairs);
                Ok(())
            }
        }
    }
}

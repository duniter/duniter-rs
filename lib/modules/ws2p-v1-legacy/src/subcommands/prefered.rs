//  Copyright (C) 2018  The Durs Project Developers.
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

//! WS2P1 module subcommand prefered

use dup_crypto::keys::PubKey;
use std::collections::HashSet;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, StructOpt)]
/// Ws2p1 prefered subcommands
pub enum Ws2pPreferedSubCommands {
    /// Add prefered pubkey
    #[structopt(
        name = "add",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Add {
        /// Public key to add
        public_keys: Vec<PubKey>,
    },
    /// Add prefered pubkeys from file (one pubkey per line)
    #[structopt(
        name = "add-file",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    AddFromFile {
        /// File path
        #[structopt(parse(from_os_str))]
        file_path: PathBuf,
    },
    /// Clear prefered pubkeys
    #[structopt(
        name = "clear",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Clear,
    /// Remove prefered pubkey
    #[structopt(
        name = "rem",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Rem {
        /// Public key to remove
        public_keys: Vec<PubKey>,
    },
    /// Show prefered pubkeys
    #[structopt(
        name = "show",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Show,
}

impl Ws2pPreferedSubCommands {
    pub fn execute(
        self,
        module_user_conf: Option<crate::WS2PUserConf>,
    ) -> Option<crate::WS2PUserConf> {
        {
            let mut prefered_pubkeys = if let Some(ref module_user_conf) = module_user_conf {
                module_user_conf
                    .prefered_pubkeys
                    .clone()
                    .unwrap_or_else(HashSet::new)
            } else {
                HashSet::new()
            };

            match self {
                Ws2pPreferedSubCommands::Add { public_keys } => {
                    for pubkey in public_keys {
                        prefered_pubkeys.insert(pubkey.to_string());
                        println!(
                            "Pubkey '{}' successfully added to the list of preferred keys.",
                            pubkey
                        );
                    }
                    let mut new_user_conf = module_user_conf.unwrap_or_default();
                    new_user_conf.prefered_pubkeys = Some(prefered_pubkeys);
                    Some(new_user_conf)
                }
                Ws2pPreferedSubCommands::AddFromFile { file_path } => {
                    if file_path.as_path().exists() {
                        match fs::File::open(file_path.as_path()) {
                            Ok(file) => {
                                let mut new_prefered_pubkeys = HashSet::new();
                                for (i, line) in std::io::BufReader::new(file).lines().enumerate() {
                                    match line {
                                        Ok(line) => match PubKey::from_str(&line) {
                                            Ok(pubkey) => {
                                                new_prefered_pubkeys.insert(pubkey.to_string());
                                                println!(
                                                            "Pubkey '{}' successfully added to the list of preferred keys.",
                                                            pubkey
                                                        );
                                            }
                                            Err(e) => {
                                                println!("Line n°{} is invalid: {}", i + 1, e);
                                            }
                                        },
                                        Err(e) => {
                                            println!("Fail to read line n°{}: {}", i + 1, e);
                                            return module_user_conf;
                                        }
                                    }
                                }
                                let mut new_user_conf = module_user_conf.unwrap_or_default();
                                if let Some(ref mut prefered_pubkeys) =
                                    new_user_conf.prefered_pubkeys
                                {
                                    prefered_pubkeys.extend(new_prefered_pubkeys.into_iter());
                                } else {
                                    new_user_conf.prefered_pubkeys = Some(new_prefered_pubkeys);
                                }
                                Some(new_user_conf)
                            }
                            Err(e) => {
                                println!("Fail to open file: {}", e);
                                module_user_conf
                            }
                        }
                    } else {
                        println!("Error: file note exist !");
                        module_user_conf
                    }
                }
                Ws2pPreferedSubCommands::Clear => {
                    if let Some(mut module_user_conf) = module_user_conf {
                        module_user_conf.prefered_pubkeys = None;
                        println!("All preferred keys removed !");
                        Some(module_user_conf)
                    } else {
                        module_user_conf
                    }
                }
                Ws2pPreferedSubCommands::Rem { public_keys } => {
                    for pubkey in public_keys {
                        prefered_pubkeys.remove(&pubkey.to_string());
                        println!(
                            "Pubkey '{}' successfully removed from the list of preferred keys",
                            pubkey
                        );
                    }
                    let mut new_user_conf = module_user_conf.unwrap_or_default();
                    new_user_conf.prefered_pubkeys = Some(prefered_pubkeys);
                    Some(new_user_conf)
                }
                Ws2pPreferedSubCommands::Show => {
                    println!("{} preferred keys: ", prefered_pubkeys.len());
                    for pubkey in &prefered_pubkeys {
                        println!("{}", pubkey);
                    }
                    module_user_conf
                }
            }
        }
    }
}

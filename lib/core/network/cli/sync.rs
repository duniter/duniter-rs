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

//! Durs network cli : sync subcommands.

extern crate structopt;

use std::str::FromStr;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "sync",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Synchronization from network
pub struct SyncOpt {
    /// The source of datas (url of the node from which to synchronize OR path to local file)
    pub source: Option<String>,
    /// The source type
    #[structopt(short = "t", long = "type", default_value = "ts")]
    pub source_type: SyncSourceType,
    /// End block
    #[structopt(short = "e", long = "end")]
    pub end: Option<u32>,
    /// cautious mode (check all protocol rules, very slow)
    #[structopt(short = "c", long = "cautious")]
    pub cautious_mode: bool,
    /// unsafe mode (not check blocks inner hashs, very dangerous)
    #[structopt(short = "u", long = "unsafe")]
    pub unsafe_mode: bool,
}

/// The source of blocks datas
#[derive(Debug, Copy, Clone)]
pub enum SyncSourceType {
    /// Sync from network
    Network,
    /// Sync from Duniter-ts sqlite bdd
    TsSqlDb,
    /// Sync from json blocks in files
    JsonFiles,
}

impl FromStr for SyncSourceType {
    type Err = String;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source {
            "n" | "network" => Ok(SyncSourceType::Network),
            "ts" | "ts-sql" => Ok(SyncSourceType::TsSqlDb),
            "json" => Ok(SyncSourceType::JsonFiles),
            &_ => Err("Unknown source type".to_owned()),
        }
    }
}

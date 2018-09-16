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

//! Durs-core cli : sync subcommands.

extern crate structopt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "sync",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// synchronization from network
pub struct SyncOpt {
    /// The domain name or ip address of the node from which to synchronize.
    pub host: String,
    /// The port number of the node from which to synchronize.
    pub port: u16,
    /// The endpoint path of the node from which to synchronize.
    pub path: Option<String>,
    #[structopt(short = "c", long = "cautious")]
    /// cautious mode (check all protocol rules, very slow)
    pub cautious_mode: bool,
    #[structopt(short = "u", long = "unsafe")]
    /// unsafe mode (not check blocks inner hashs, very dangerous)
    pub unsafe_mode: bool,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(
    name = "sync",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// synchronization via a duniter-ts database
pub struct SyncTsOpt {
    /// Set the ts profile to use
    pub ts_profile: Option<String>,
    #[structopt(short = "c", long = "cautious")]
    /// cautious mode (check all protocol rules, very slow)
    pub cautious_mode: bool,
    #[structopt(short = "u", long = "unsafe")]
    /// unsafe mode (not check blocks inner hashs, very dangerous)
    pub unsafe_mode: bool,
}

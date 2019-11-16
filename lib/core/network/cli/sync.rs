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

//! Dunitrust network cli : sync subcommands.

use durs_network_documents::url::Url;
use std::path::PathBuf;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "sync", setting(structopt::clap::AppSettings::ColoredHelp))]
/// Synchronization from network
pub struct SyncOpt {
    /// cautious mode (check all protocol rules, very slow)
    #[structopt(long = "cautious")]
    pub cautious_mode: bool,
    /// Currency
    #[structopt(short = "c", long = "currency")]
    pub currency: Option<String>,
    /// End block
    #[structopt(short = "e", long = "end")]
    pub end: Option<u32>,
    /// Path to directory that contain blockchain json files
    #[structopt(short = "l", long = "local")]
    #[structopt(parse(from_os_str))]
    pub local_path: Option<PathBuf>,
    /// The source of datas (url of the node from which to synchronize)
    pub source: Option<Url>,
    /// Start node after sync (not yet implemented)
    #[structopt(short = "s", long = "start", hidden = true)]
    pub start: bool,
    /// Sync module name
    #[structopt(short = "m", long = "sync-module")]
    pub sync_module_name: Option<String>,
    /// unsafe mode (not check blocks inner hashs, very dangerous)
    #[structopt(short = "u", long = "unsafe", hidden = true)]
    pub unsafe_mode: bool,
}

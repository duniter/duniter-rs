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

//! WS2P1 module subcommands

pub mod prefered;

use prefered::Ws2pPreferedSubCommands;

#[derive(Clone, Debug, StructOpt)]
/// Ws2p1 subcommands
pub enum WS2PSubCommands {
    /// Prefered keys
    #[structopt(
        name = "prefered",
        raw(setting = "structopt::clap::AppSettings::ColoredHelp")
    )]
    Prefered {
        #[structopt(subcommand)]
        subcommand: Ws2pPreferedSubCommands,
    },
}

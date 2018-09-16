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

//! Main function for classic duniter-rust nodes (no specialization).

#![cfg_attr(feature = "strict", deny(warnings))]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate duniter_core;
#[cfg(unix)]
extern crate duniter_tui;
extern crate durs_ws2p_v1_legacy;
//extern crate durs_ws2p;
extern crate structopt;

pub use duniter_core::{cli::DursOpt, DuRsConf, DuniterCore, UserCommand};
#[cfg(unix)]
pub use duniter_tui::TuiModule;
pub use durs_ws2p_v1_legacy::WS2PModule;
//pub use durs_ws2p::WS2Pv2Module;
use structopt::StructOpt;

/// Main function
#[cfg(unix)]
#[cfg(not(target_arch = "arm"))]
fn main() {
    // Get software name and version
    let soft_name = env!("CARGO_PKG_NAME");
    let soft_version = env!("CARGO_PKG_VERSION");

    // Instantiate duniter core
    let clap_app = DursOpt::clap();
    let mut duniter_core = DuniterCore::<DuRsConf>::new(soft_name, soft_version, &clap_app, 0);

    // Inject plugins subcommands
    //duniter_core.inject_cli_subcommand::<GvaModule>();
    duniter_core.inject_cli_subcommand::<TuiModule>();
    duniter_core.inject_cli_subcommand::<WS2PModule>();

    // Match user command
    if duniter_core.match_user_command() {
        // Plug all plugins
        //duniter_core.plug::<GuiModule>();
        //duniter_core.plug::<GvaModule>();
        //duniter_core.plug::<PoolModule>();
        //duniter_core.plug::<PowModule>();
        duniter_core.plug::<TuiModule>();
        duniter_core.plug_network::<WS2PModule>();
        duniter_core.start_core();
    }
}
#[cfg(unix)]
#[cfg(target_arch = "arm")]
fn main() {
    // Get software name and version
    let soft_name = env!("CARGO_PKG_NAME");
    let soft_version = env!("CARGO_PKG_VERSION");

    // Instantiate duniter core
    let clap_app = DursOpt::clap();
    let mut duniter_core = DuniterCore::<DuRsConf>::new(soft_name, soft_version, &clap_app, 0);

    // Inject plugins subcommands
    //duniter_core.inject_cli_subcommand::<DasaModule>();
    //duniter_core.inject_cli_subcommand::<GvaModule>();
    duniter_core.inject_cli_subcommand::<TuiModule>();
    duniter_core.inject_cli_subcommand::<WS2PModule>();

    // Match user command
    if duniter_core.match_user_command() {
        // Plug all plugins
        //duniter_core.plug::<DasaModule>();
        //duniter_core.plug::<GuiModule>();
        //duniter_core.plug::<GvaModule>();
        //duniter_core.plug::<PoolModule>();
        //duniter_core.plug::<PowModule>();
        duniter_core.plug::<TuiModule>();
        duniter_core.plug_network::<WS2PModule>();
        duniter_core.start_core();
    }
}
#[cfg(windows)]
fn main() {
    // Get software name and version
    let soft_name = env!("CARGO_PKG_NAME");
    let soft_version = env!("CARGO_PKG_VERSION");

    // Instantiate duniter core
    let clap_app = DursOpt::clap();
    let mut duniter_core = DuniterCore::<DuRsConf>::new(soft_name, soft_version, &clap_app, 0);

    // Inject plugins subcommands
    //duniter_core.inject_cli_subcommand::<GvaModule>();
    duniter_core.inject_cli_subcommand::<WS2PModule>();

    // Match user command
    if duniter_core.match_user_command() {
        // Plug all plugins
        //duniter_core.plug::<DasaModule>();
        //duniter_core.plug::<GuiModule>();
        //duniter_core.plug::<GvaModule>();
        //duniter_core.plug::<PoolModule>();
        //duniter_core.plug::<PowModule>();
        duniter_core.plug_network::<WS2PModule>();
        duniter_core.start_core();
    }
}

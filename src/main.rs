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
    duniter_core::main(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        &DursOpt::clap(),
        |core| {
            //core.inject_cli_subcommand::<DasaModule>();
            core.inject_cli_subcommand::<TuiModule>();
            core.inject_cli_subcommand::<WS2PModule>();
        },
        |core| {
            //core.inject_cli_subcommand::<DasaModule>();
            core.plug::<TuiModule>();
            core.plug_network::<WS2PModule>();
        },
    );
}
#[cfg(unix)]
#[cfg(target_arch = "arm")]
fn main() {
    duniter_core::main(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        &DursOpt::clap(),
        |core| {
            core.inject_cli_subcommand::<TuiModule>();
            core.inject_cli_subcommand::<WS2PModule>();
        },
        |core| {
            core.plug::<TuiModule>();
            core.plug_network::<WS2PModule>();
        },
    );
}
#[cfg(windows)]
fn main() {
    duniter_core::main(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        &DursOpt::clap(),
        |core| {
            core.inject_cli_subcommand::<WS2PModule>();
        },
        |core| {
            core.plug_network::<WS2PModule>();
        },
    );
}

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

//! Main function for classic Dunitrust nodes (no specialization).

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

pub mod cli;
mod init;

use crate::cli::DursCliOpt;
use crate::init::init;
use durs_core::durs_plug;
#[cfg(not(target_arch = "arm"))]
pub use durs_gva::GvaModule;
#[cfg(unix)]
pub use durs_tui::TuiModule;
use log::error;
use structopt::StructOpt;
//pub use durs_skeleton::SkeletonModule;
pub use durs_ws2p::WS2PModule;
pub use durs_ws2p_v1_legacy::WS2Pv1Module;

/// Dunitrust cli main macro
macro_rules! durs_cli_main {
    ( $closure_plug:expr ) => {{
        init();
        if let Err(err) = DursCliOpt::from_args().into_durs_command().execute(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            $closure_plug,
        ) {
            println!("{}", err);
            error!("{}", err);
        }
    }};
}

/// Dunitrust command line edition, main function
#[cfg(unix)]
#[cfg(not(target_arch = "arm"))]
fn main() {
    durs_cli_main!(durs_plug!(
        [WS2Pv1Module, WS2PModule],
        [TuiModule, GvaModule /*, SkeletonModule ,DasaModule*/]
    ))
}
#[cfg(unix)]
#[cfg(target_arch = "arm")]
fn main() {
    durs_cli_main!(durs_plug!(
        [WS2Pv1Module, WS2PModule],
        [TuiModule /*, SkeletonModule*/]
    ))
}
#[cfg(windows)]
fn main() {
    durs_cli_main!(durs_plug!([WS2Pv1Module, WS2PModule], []))
}

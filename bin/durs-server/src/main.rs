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

//! Main function for classic Durs nodes (no specialization).

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
use log::error;
use structopt::StructOpt;

#[cfg(unix)]
pub use durs_tui::TuiModule;
//pub use durs_skeleton::SkeletonModule;
pub use durs_ws2p_v1_legacy::{WS2PModule, WS2POpt};
//pub use durs_ws2p::WS2Pv2Module;

/// Durs command line edition, main function
#[cfg(unix)]
#[cfg(not(target_arch = "arm"))]
fn main() {
    init();
    if let Err(err) = DursCliOpt::from_args()
        .into_durs_command()
        .execute(durs_plug!(
            [WS2PModule],
            [TuiModule /*, SkeletonModule ,DasaModule*/]
        ))
    {
        println!("{}", err);
        error!("{}", err);
    }
}
#[cfg(unix)]
#[cfg(target_arch = "arm")]
fn main() {
    init();
    if let Err(err) = DursCliOpt::from_args()
        .into_durs_command()
        .execute(durs_plug!([WS2PModule], [TuiModule /*, SkeletonModule*/]))
    {
        println!("{}", err);
        error!("{}", err);
    }
}
#[cfg(windows)]
fn main() {
    init();
    if let Err(err) = DursCliOpt::from_args()
        .into_durs_command()
        .execute(durs_plug!([WS2PModule], []))
    {
        println!("{}", err);
        error!("{}", err);
    }
}

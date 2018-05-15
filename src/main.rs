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
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate duniter_core;
extern crate duniter_tui;
extern crate duniter_ws2p;

pub use duniter_core::DuniterCore;
pub use duniter_tui::TuiModule;
pub use duniter_ws2p::WS2PModule;

/// Main function
fn main() {
    // Get software name and version
    let soft_name = env!("CARGO_PKG_NAME");
    let soft_version = env!("CARGO_PKG_VERSION");

    // Run duniter core
    if let Some(mut duniter_core) = DuniterCore::new(soft_name, soft_version) {
        duniter_core.plug::<WS2PModule>();
        duniter_core.plug::<TuiModule>();
        //duniter_core.plug::<PoolModule>();
        //duniter_core.plug::<PowModule>();
        //duniter_core.plug::<GvaModule>();
        //duniter_core.plug::<DasaModule>();
        //duniter_core.plug::<GuiModule>();
        duniter_core.start_blockchain();
    };
}

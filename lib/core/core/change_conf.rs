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

//! Crate containing Duniter-rust core.

use durs_conf::ChangeGlobalConf;
use durs_module::DursConfTrait;

/// Change global configuration
pub fn change_global_conf<DC: DursConfTrait>(
    profile: &str,
    mut conf: DC,
    user_request: ChangeGlobalConf,
) {
    match user_request {
        ChangeGlobalConf::ChangeCurrency(_) => {}
        ChangeGlobalConf::DisableModule(module_id) => conf.disable(module_id),
        ChangeGlobalConf::EnableModule(module_id) => conf.enable(module_id),
        ChangeGlobalConf::None() => {}
    }

    // Write new conf
    durs_conf::write_conf_file(&durs_conf::get_conf_path(profile), &conf)
        .expect("IOError : Fail to update conf  ");

    println!("Configuration successfully updated.");
}

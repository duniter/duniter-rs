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

//! Crate containing Duniter-rust core.

use crate::errors::DursCoreError;
use durs_conf::ChangeGlobalConf;
use durs_module::DursConfTrait;
use std::path::PathBuf;

/// Change global configuration
pub fn change_global_conf<DC: DursConfTrait>(
    profile_path: &PathBuf,
    conf: &mut DC,
    user_request: ChangeGlobalConf,
) -> Result<(), DursCoreError> {
    match user_request {
        ChangeGlobalConf::ChangeCurrency(_) => {}
        ChangeGlobalConf::DisableModule(module_id) => conf.disable(module_id),
        ChangeGlobalConf::EnableModule(module_id) => conf.enable(module_id),
        ChangeGlobalConf::None() => {}
    }

    // Write new conf
    durs_conf::file::write_conf_file(&durs_conf::file::get_conf_path(profile_path), conf)
        .map_err(DursCoreError::FailUpdateConf)
}

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

//! Dunitrust global configuration

pub mod v2;

use durs_common_tools::fatal_error;
use durs_module::{DursGlobalConfTrait, ModuleName};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Dunitrust global configuration (without modules configuration)
pub enum DuRsGlobalConf {
    /// Dunitrust global configuration v1
    V1(crate::v1::DuRsConfV1),
    /// Dunitrust global configuration v2
    V2(v2::DuRsGlobalConfV2),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
/// Dunitrust global configuration (without modules configuration)
pub enum DuRsGlobalUserConf {
    /// Dunitrust global user configuration v2
    V2(v2::DuRsGlobalUserConfV2),
}

impl DursGlobalConfTrait for DuRsGlobalConf {
    type GlobalUserConf = DuRsGlobalUserConf;

    fn my_node_id(&self) -> u32 {
        match *self {
            DuRsGlobalConf::V1(ref conf_v1) => conf_v1.my_node_id,
            DuRsGlobalConf::V2(ref conf_v2) => conf_v2.my_node_id,
        }
    }
    fn default_sync_module(&self) -> ModuleName {
        match *self {
            DuRsGlobalConf::V1(_) => {
                fatal_error!("Feature default_sync_module not exist in durs conf v1 !")
            }
            DuRsGlobalConf::V2(ref conf_v2) => conf_v2.default_sync_module.clone(),
        }
    }
}

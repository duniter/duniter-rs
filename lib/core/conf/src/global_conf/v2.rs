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

//! Dunitrust global configuration V2

use crate::constants;
use crate::resources::ResourcesUsage;
use crate::v1::DuRsConfV1;
use dubp_currency_params::CurrencyName;
use durs_module::ModuleName;
use std::collections::HashSet;

#[derive(Debug, Default, Clone, Deserialize, PartialEq, Serialize)]
/// Dunitrust configuration v2
pub struct DuRsGlobalUserConfV2 {
    /// Currency name
    pub currency: Option<CurrencyName>,
    /// Node unique identifier
    pub my_node_id: Option<u32>,
    /// Name of the module used by default for synchronization
    pub default_sync_module: Option<ModuleName>,
    /// Ressources usage
    pub resources_usage: Option<ResourcesUsage>,
    /// Disabled modules
    pub disabled: Option<HashSet<ModuleName>>,
    /// Enabled modules
    pub enabled: Option<HashSet<ModuleName>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Dunitrust configuration v2
pub struct DuRsGlobalConfV2 {
    /// Currency name
    pub currency: CurrencyName,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Name of the module used by default for synchronization
    pub default_sync_module: ModuleName,
    /// Ressources usage
    pub resources_usage: ResourcesUsage,
    /// Disabled modules
    pub disabled: HashSet<ModuleName>,
    /// Enabled modules
    pub enabled: HashSet<ModuleName>,
}

impl Default for DuRsGlobalConfV2 {
    fn default() -> Self {
        DuRsGlobalConfV2 {
            currency: CurrencyName(String::from(constants::DEFAULT_CURRENCY)),
            my_node_id: crate::generate_random_node_id(),
            default_sync_module: ModuleName(String::from(constants::DEFAULT_DEFAULT_SYNC_MODULE)),
            resources_usage: ResourcesUsage::default(),
            disabled: HashSet::with_capacity(0),
            enabled: HashSet::with_capacity(0),
        }
    }
}

impl From<DuRsConfV1> for DuRsGlobalConfV2 {
    fn from(conf_v1: DuRsConfV1) -> Self {
        DuRsGlobalConfV2 {
            currency: conf_v1.currency,
            my_node_id: conf_v1.my_node_id,
            default_sync_module: ModuleName(String::from(constants::DEFAULT_DEFAULT_SYNC_MODULE)),
            resources_usage: ResourcesUsage::default(),
            disabled: conf_v1.disabled,
            enabled: conf_v1.enabled,
        }
    }
}

impl DuRsGlobalConfV2 {
    /// Override configuration with user configuration
    pub fn r#override(self, global_user_conf: DuRsGlobalUserConfV2) -> Self {
        DuRsGlobalConfV2 {
            currency: global_user_conf.currency.unwrap_or(self.currency),
            my_node_id: global_user_conf.my_node_id.unwrap_or(self.my_node_id),
            default_sync_module: global_user_conf
                .default_sync_module
                .unwrap_or(self.default_sync_module),
            resources_usage: global_user_conf
                .resources_usage
                .unwrap_or(self.resources_usage),
            disabled: global_user_conf.disabled.unwrap_or(self.disabled),
            enabled: global_user_conf.enabled.unwrap_or(self.enabled),
        }
    }
}

impl From<DuRsGlobalUserConfV2> for DuRsGlobalConfV2 {
    fn from(global_user_conf: DuRsGlobalUserConfV2) -> Self {
        Self::default().r#override(global_user_conf)
    }
}

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

//! Dunitrust configuration v1

use crate::modules_conf::ModulesConf;
use dubp_currency_params::CurrencyName;
use durs_module::ModuleName;
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Duniter configuration v1
pub struct DuRsConfV1 {
    /// Currency name
    pub currency: CurrencyName,
    /// Duniter node unique identifier
    pub my_node_id: u32,
    /// Configuration of modules in json format (obtained from the conf.json file)
    pub modules: ModulesConf,
    /// Disabled modules
    pub disabled: HashSet<ModuleName>,
    /// Enabled modules
    pub enabled: HashSet<ModuleName>,
}

impl Default for DuRsConfV1 {
    fn default() -> Self {
        DuRsConfV1 {
            currency: CurrencyName(String::from(crate::constants::DEFAULT_CURRENCY)),
            my_node_id: crate::generate_random_node_id(),
            modules: ModulesConf::default(),
            disabled: HashSet::with_capacity(0),
            enabled: HashSet::with_capacity(0),
        }
    }
}

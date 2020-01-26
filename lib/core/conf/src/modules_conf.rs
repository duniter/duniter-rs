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

//! Dunitrust modules configuration

use durs_module::ModuleName;

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
/// Modules conf
pub struct ModulesConf(pub serde_json::Value);

impl Default for ModulesConf {
    #[inline]
    fn default() -> Self {
        ModulesConf(serde_json::Value::Null)
    }
}

impl ModulesConf {
    /// Change module conf
    pub fn set_module_conf(&mut self, module_name: ModuleName, new_module_conf: serde_json::Value) {
        if self.0.is_null() {
            let mut new_modules_conf = serde_json::Map::with_capacity(1);
            new_modules_conf.insert(module_name.0, new_module_conf);
            self.0 = serde_json::value::to_value(new_modules_conf)
                .expect("Fail to create map of new modules conf !");
        } else {
            self.0
                .as_object_mut()
                .expect("Conf file currupted !")
                .insert(module_name.0, new_module_conf);
        }
    }
}

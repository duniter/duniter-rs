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

use crate::constants;
use crate::keypairs::DuniterKeyPairs;
use crate::DuRsConf;
use dubp_currency_params::CurrencyName;
use durs_common_tools::traits::merge::Merge;
use durs_message::DursMsg;
use durs_module::{
    DursConfTrait, DursModule, ModuleConfError, ModuleName, ModuleStaticName, RequiredKeysContent,
};

/// Module configurations and required keys
pub type ModuleConfsAndKeys<M> = (
    (
        <M as DursModule<DuRsConf, DursMsg>>::ModuleConf,
        Option<<M as DursModule<DuRsConf, DursMsg>>::ModuleUserConf>,
    ),
    RequiredKeysContent,
);

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
    // get module conf
    fn get_module_conf<M: DursModule<DuRsConf, DursMsg>>(
        currency_name: Option<&CurrencyName>,
        global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
        module_conf_json: Option<serde_json::Value>,
    ) -> Result<(M::ModuleConf, Option<M::ModuleUserConf>), ModuleConfError> {
        let file_module_user_conf: M::ModuleUserConf =
            if let Some(module_conf_json) = module_conf_json {
                let file_module_user_conf_opt: Option<M::ModuleUserConf> =
                    serde_json::from_str(module_conf_json.to_string().as_str())?;
                file_module_user_conf_opt.unwrap_or_default()
            } else {
                M::ModuleUserConf::default()
            };

        let env_module_user_conf = Self::get_env_module_user_conf::<M::ModuleUserConf>(M::name())?;

        M::generate_module_conf(
            currency_name,
            global_conf,
            Some(file_module_user_conf.merge(env_module_user_conf)),
        )
    }

    // get module conf from environment variables
    fn get_env_module_user_conf<ModuleUserConf: serde::de::DeserializeOwned>(
        module_name: ModuleStaticName,
    ) -> Result<ModuleUserConf, ModuleConfError> {
        let prefix = format!(
            "{}{}_",
            constants::DURS_ENV_PREFIX,
            module_name.0.to_ascii_uppercase()
        );

        envy::prefixed(prefix)
            .from_env::<ModuleUserConf>()
            .map_err(ModuleConfError::EnvyErr)
    }
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

/// Get module conf and keys
pub fn get_module_conf_and_keys<M: DursModule<DuRsConf, DursMsg>>(
    currency_name: Option<&CurrencyName>,
    global_conf: &<DuRsConf as DursConfTrait>::GlobalConf,
    module_conf_json: Option<serde_json::Value>,
    keypairs: DuniterKeyPairs,
) -> Result<ModuleConfsAndKeys<M>, ModuleConfError> {
    Ok((
        ModulesConf::get_module_conf::<M>(currency_name, global_conf, module_conf_json)?,
        DuniterKeyPairs::get_required_keys_content(M::ask_required_keys(), keypairs),
    ))
}

#[cfg(test)]
mod tests {

    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    // Empty mutex used to ensure that only one test runs at a time
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct TestModuleUserConf {
        field1: Option<String>,
        field2: Option<usize>,
    }

    #[inline]
    fn prefix() -> String {
        format!("{}MODULE_TEST_", constants::DURS_ENV_PREFIX)
    }

    fn clear_env_vars() {
        if std::env::var(&format!("{}FIELD1", prefix())).is_ok() {
            std::env::remove_var(&format!("{}FIELD1", prefix()));
        }
        if std::env::var(&format!("{}FIELD2", prefix())).is_ok() {
            std::env::remove_var(&format!("{}FIELD2", prefix()));
        }
    }

    #[test]
    fn test_env_module_conf_without_env_vars() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        assert_eq!(
            TestModuleUserConf::default(),
            ModulesConf::get_env_module_user_conf(ModuleStaticName("module_test"))?,
        );

        Ok(())
    }

    #[test]
    fn test_env_module_conf_with_some_valid_env_vars() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        std::env::set_var(&format!("{}FIELD1", prefix()), "toto");
        std::env::set_var(&format!("{}FIELD2", prefix()), "4");

        assert_eq!(
            TestModuleUserConf {
                field1: Some("toto".to_owned()),
                field2: Some(4),
            },
            ModulesConf::get_env_module_user_conf(ModuleStaticName("module_test"))?,
        );

        Ok(())
    }

    #[test]
    fn test_env_module_conf_with_invalid_env_var() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        // field2 must be a number
        std::env::set_var(&format!("{}FIELD2", prefix()), "toto");

        if let Err(ModuleConfError::EnvyErr(_)) = ModulesConf::get_env_module_user_conf::<
            TestModuleUserConf,
        >(ModuleStaticName("module_test"))
        {
            Ok(())
        } else {
            panic!("get_env_module_user_conf() must return an error ModuleConfError::EnvyErr.");
        }
    }
}

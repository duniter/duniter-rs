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
                serde_json::from_str(module_conf_json.to_string().as_str())?
            } else {
                M::ModuleUserConf::default()
            };

        let env_module_user_conf = Self::get_env_module_user_conf::<M::ModuleUserConf>(M::name())?;

        M::generate_module_conf(
            currency_name,
            global_conf,
            Some(env_module_user_conf.merge(file_module_user_conf)),
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
    use crate::global_conf::v2::DuRsGlobalConfV2;
    use crate::global_conf::DuRsGlobalConf;
    use dup_crypto::keys::{ed25519, KeyPairEnum};
    use durs_module::module_test::*;
    use once_cell::sync::Lazy;
    use serde_json::json;
    use std::sync::Mutex;

    // Empty mutex used to ensure that only one test runs at a time
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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

    fn keypairs() -> DuniterKeyPairs {
        DuniterKeyPairs {
            network_keypair: KeyPairEnum::Ed25519(
                ed25519::Ed25519KeyPair::generate_random().expect("unspecified rand error"),
            ),
            member_keypair: None,
        }
    }

    #[test]
    fn test_get_empty_module_conf() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        let (confs, keys): ModuleConfsAndKeys<ModuleTest<DuRsConf, DursMsg>> =
            get_module_conf_and_keys::<ModuleTest<DuRsConf, DursMsg>>(
                None,
                &DuRsGlobalConf::V2(DuRsGlobalConfV2::default()),
                None,
                keypairs(),
            )?;

        assert_eq!(
            (
                ModuleTestConf::default(),
                Some(ModuleTestUserConf::default())
            ),
            confs,
        );
        assert_eq!(RequiredKeysContent::None, keys,);

        Ok(())
    }

    #[test]
    fn test_get_module_conf_from_file() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        let json_conf = json!({
            "field1": "toto",
            "field2": 123,
        });

        let (confs, _): ModuleConfsAndKeys<ModuleTest<DuRsConf, DursMsg>> =
            get_module_conf_and_keys::<ModuleTest<DuRsConf, DursMsg>>(
                None,
                &DuRsGlobalConf::V2(DuRsGlobalConfV2::default()),
                Some(json_conf),
                keypairs(),
            )?;

        assert_eq!(
            (
                ModuleTestConf {
                    field1: "toto".to_owned(),
                    field2: 123,
                },
                Some(ModuleTestUserConf {
                    field1: Some("toto".to_owned()),
                    field2: Some(123),
                })
            ),
            confs,
        );

        Ok(())
    }

    #[test]
    fn test_get_module_conf_from_env_and_file() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        std::env::set_var(&format!("{}FIELD2", prefix()), "456");

        let json_conf = json!({
            "field1": "toto",
            "field2": 123,
        });

        let (confs, _): ModuleConfsAndKeys<ModuleTest<DuRsConf, DursMsg>> =
            get_module_conf_and_keys::<ModuleTest<DuRsConf, DursMsg>>(
                None,
                &DuRsGlobalConf::V2(DuRsGlobalConfV2::default()),
                Some(json_conf),
                keypairs(),
            )?;

        assert_eq!(
            (
                ModuleTestConf {
                    field1: "toto".to_owned(),
                    field2: 456,
                },
                Some(ModuleTestUserConf {
                    field1: Some("toto".to_owned()),
                    field2: Some(456),
                })
            ),
            confs,
        );

        Ok(())
    }

    #[test]
    fn test_env_module_conf_without_env_vars() -> Result<(), ModuleConfError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");
        clear_env_vars();

        assert_eq!(
            ModuleTestUserConf::default(),
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
            ModuleTestUserConf {
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
            ModuleTestUserConf,
        >(ModuleStaticName("module_test"))
        {
            Ok(())
        } else {
            panic!("get_env_module_user_conf() must return an error ModuleConfError::EnvyErr.");
        }
    }
}

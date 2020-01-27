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

//! Dunitrust configuration from environment variables

use crate::constants;
use crate::errors::DursConfEnvError;
use crate::global_conf::v2::DuRsGlobalUserConfV2;
use crate::global_conf::DuRsGlobalUserConf;
use crate::resources::ResourcesUsage;

/// Load global user configuration from environment variables
pub fn load_env_global_user_conf() -> Result<DuRsGlobalUserConf, DursConfEnvError> {
    if let Ok(conf_version) = std::env::var(constants::DURS_CONF_VERSION) {
        match conf_version
            .parse::<usize>()
            .map_err(DursConfEnvError::ConfVersionParseErr)?
        {
            2 => {
                let resources_usage =
                    envy::prefixed(&format!("{}RESOURCES_USAGE_", constants::DURS_ENV_PREFIX))
                        .from_env::<ResourcesUsage>()
                        .map_err(DursConfEnvError::EnvyErr)?;
                let mut global_user_conf_v2 = envy::prefixed(constants::DURS_ENV_PREFIX)
                    .from_env::<DuRsGlobalUserConfV2>()
                    .map_err(DursConfEnvError::EnvyErr)?;
                global_user_conf_v2.resources_usage = Some(resources_usage);
                Ok(DuRsGlobalUserConf::V2(global_user_conf_v2))
            }
            v => Err(DursConfEnvError::UnsupportedVersion {
                expected: vec![2],
                found: v,
            }),
        }
    } else {
        Ok(DuRsGlobalUserConf::V2(DuRsGlobalUserConfV2::default()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::resources::ResourceUsage;
    use dubp_currency_params::CurrencyName;
    use durs_module::ModuleName;
    use maplit::hashset;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    // Empty mutex used to ensure that only one test runs at a time
    static MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_env_conf_without_env_vars() -> Result<(), DursConfEnvError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");

        std::env::remove_var(constants::DURS_CONF_VERSION);

        assert_eq!(
            DuRsGlobalUserConf::V2(DuRsGlobalUserConfV2::default()),
            load_env_global_user_conf()?,
        );

        Ok(())
    }

    #[test]
    fn test_env_conf_with_unsupported_conf_version_var() -> Result<(), DursConfEnvError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");

        std::env::set_var(constants::DURS_CONF_VERSION, "3");

        if let Err(DursConfEnvError::UnsupportedVersion { .. }) = load_env_global_user_conf() {
            Ok(())
        } else {
            panic!("load_env_global_user_conf() must return an error DursConfEnvError::UnsupportedVersion.");
        }
    }

    #[test]
    fn test_env_conf_with_some_valid_env_vars() -> Result<(), DursConfEnvError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");

        std::env::set_var(constants::DURS_CONF_VERSION, "2");
        std::env::set_var(&format!("{}CURRENCY", constants::DURS_ENV_PREFIX), "g1");
        std::env::set_var(
            &format!("{}DISABLED", constants::DURS_ENV_PREFIX),
            "tui,gva",
        );
        std::env::set_var(
            &format!("{}RESOURCES_USAGE_MEMORY_USAGE", constants::DURS_ENV_PREFIX),
            "medium",
        );

        assert_eq!(
            DuRsGlobalUserConf::V2(DuRsGlobalUserConfV2 {
                currency: Some(CurrencyName(String::from("g1"))),
                my_node_id: None,
                default_sync_module: None,
                resources_usage: Some(ResourcesUsage {
                    cpu_usage: ResourceUsage::Large,
                    network_usage: ResourceUsage::Large,
                    memory_usage: ResourceUsage::Medium,
                    disk_space_usage: ResourceUsage::Large,
                }),
                disabled: Some(hashset![
                    ModuleName("tui".to_owned()),
                    ModuleName("gva".to_owned())
                ]),
                enabled: None,
            }),
            load_env_global_user_conf()?,
        );

        Ok(())
    }

    #[test]
    fn test_env_conf_with_invalid_conf_version_var() -> Result<(), DursConfEnvError> {
        let _lock = MUTEX.lock().expect("MUTEX poisoned");

        std::env::set_var(constants::DURS_CONF_VERSION, "str");

        if let Err(DursConfEnvError::ConfVersionParseErr(_)) = load_env_global_user_conf() {
            Ok(())
        } else {
            panic!("load_env_global_user_conf() must return an error DursConfEnvError::ConfVersionParseErr.");
        }
    }
}

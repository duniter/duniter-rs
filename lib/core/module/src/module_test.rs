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

//! Define a test module for automated tests

use crate::*;
use std::marker::PhantomData;

/// Module test
#[derive(Debug)]
pub struct ModuleTest<DC: DursConfTrait, M: ModuleMessage> {
    phantom: PhantomData<DC>,
    phantom2: PhantomData<M>,
}

/// Module test user config
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ModuleTestUserConf {
    /// Field 1
    pub field1: Option<String>,
    /// Field 2
    pub field2: Option<usize>,
}

impl Merge for ModuleTestUserConf {
    fn merge(self, other: Self) -> Self {
        ModuleTestUserConf {
            field1: self.field1.or(other.field1),
            field2: self.field2.or(other.field2),
        }
    }
}

/// Module test config
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ModuleTestConf {
    /// Field 1
    pub field1: String,
    /// Field 2
    pub field2: usize,
}

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "test", setting(structopt::clap::AppSettings::ColoredHelp))]
/// Gva subcommand options
pub struct ModuleTestOpt {
    /// Field
    #[structopt(long = "field")]
    pub field: Option<String>,
}

impl<DC: DursConfTrait, M: ModuleMessage> DursModule<DC, M> for ModuleTest<DC, M> {
    type ModuleConf = ModuleTestConf;
    type ModuleUserConf = ModuleTestUserConf;
    type ModuleOpt = ModuleTestOpt;

    fn name() -> ModuleStaticName {
        ModuleStaticName("module_test")
    }
    fn priority() -> ModulePriority {
        ModulePriority::Recommended
    }
    fn ask_required_keys() -> RequiredKeys {
        RequiredKeys::None
    }
    fn have_subcommand() -> bool {
        false
    }
    fn generate_module_conf(
        _currency_name: Option<&CurrencyName>,
        _global_conf: &<DC as DursConfTrait>::GlobalConf,
        module_user_conf_opt: Option<Self::ModuleUserConf>,
    ) -> Result<(Self::ModuleConf, Option<Self::ModuleUserConf>), ModuleConfError> {
        let module_conf = if let Some(module_user_conf) = module_user_conf_opt.clone() {
            ModuleTestConf {
                field1: module_user_conf.field1.unwrap_or_default(),
                field2: module_user_conf.field2.unwrap_or_default(),
            }
        } else {
            ModuleTestConf::default()
        };

        Ok((module_conf, module_user_conf_opt))
    }
    fn exec_subcommand(
        _soft_meta_datas: &SoftwareMetaDatas<DC>,
        _keys: RequiredKeysContent,
        _module_conf: Self::ModuleConf,
        _module_user_conf: Option<Self::ModuleUserConf>,
        _subcommand_args: Self::ModuleOpt,
    ) -> Option<Self::ModuleUserConf> {
        unimplemented!()
    }
    fn start(
        _soft_meta_datas: &SoftwareMetaDatas<DC>,
        _keys: RequiredKeysContent,
        _conf: Self::ModuleConf,
        _router_sender: std::sync::mpsc::Sender<RouterThreadMessage<M>>,
    ) -> Result<(), failure::Error> {
        unimplemented!()
    }
}

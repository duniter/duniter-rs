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

//! Durs-core cli : modules manager subcommands.

extern crate structopt;

use duniter_module::*;
use std::collections::HashSet;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "enable",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Enable some module
pub struct EnableOpt {
    #[structopt(parse(from_str))]
    /// The module name to enable
    pub module_name: ModuleId,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "disable",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// Disable some module
pub struct DisableOpt {
    #[structopt(parse(from_str))]
    /// The module name to disable
    pub module_name: ModuleId,
}

#[derive(StructOpt, Debug, Copy, Clone)]
#[structopt(
    name = "modules",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
/// list module
pub struct ListModulesOpt {
    #[structopt(short = "d")]
    /// list only disabled modules
    pub disabled: bool,
    #[structopt(short = "e")]
    /// list only enabled modules
    pub enabled: bool,
    #[structopt(short = "n")]
    /// list only network modules
    pub network: bool,
    #[structopt(short = "s")]
    /// list only modules having access to the secret member key
    pub secret: bool,
}

impl ListModulesOpt {
    /// Extract modules filters from cli options
    pub fn get_filters(self) -> HashSet<ModulesFilter> {
        let mut filters = HashSet::with_capacity(4);
        if self.disabled {
            filters.insert(ModulesFilter::Enabled(false));
        }
        if self.enabled {
            filters.insert(ModulesFilter::Enabled(true));
        }
        if self.network {
            filters.insert(ModulesFilter::Network());
        }
        if self.secret {
            filters.insert(ModulesFilter::RequireMemberPrivKey());
        }
        filters
    }
}

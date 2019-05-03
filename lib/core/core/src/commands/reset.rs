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

//! Durs-core cli : reset subcommand.

use super::InvalidInput;
use crate::commands::DursExecutableCoreCommand;
use crate::errors::DursCoreError;
use crate::DursCore;
use durs_conf::DuRsConf;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
/// Reset type
pub enum ResetType {
    /// Reset datas
    Datas,
    /// Reset configuration
    Conf,
    /// Reset all
    All,
}

impl FromStr for ResetType {
    type Err = InvalidInput;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source {
            "data" => Ok(ResetType::Datas),
            "conf" => Ok(ResetType::Conf),
            "all" => Ok(ResetType::All),
            _ => Err(InvalidInput("Kind of data to be reseted: data, conf, all.")),
        }
    }
}

#[derive(StructOpt, Debug, Copy, Clone)]
/// Reset data or configuration
pub struct ResetOpt {
    /// Kind of data to be reseted: data, conf, all
    pub reset_type: ResetType,
}

impl DursExecutableCoreCommand for ResetOpt {
    fn execute(self, durs_core: DursCore<DuRsConf>) -> Result<(), DursCoreError> {
        let profile_path = durs_core.soft_meta_datas.profile_path;

        match self.reset_type {
            ResetType::Datas => {
                let mut currency_datas_path = profile_path;
                currency_datas_path.push("g1");
                fs::remove_dir_all(currency_datas_path.as_path())
                    .map_err(DursCoreError::FailRemoveDatasDir)
            }
            ResetType::Conf => {
                let mut conf_file_path = profile_path.clone();
                conf_file_path.push("conf.json");
                fs::remove_file(conf_file_path.as_path()).map_err(DursCoreError::FailRemoveConfFile)
            }
            ResetType::All => fs::remove_dir_all(profile_path.as_path())
                .map_err(DursCoreError::FailRemoveProfileDir),
        }
    }
}

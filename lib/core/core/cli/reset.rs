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

use crate::cli::InvalidInput;
use std::str::FromStr;

#[derive(StructOpt, Debug, Copy, Clone)]
/// Reset data or configuration
pub struct ResetOpt {
    /// Kind of data to be reseted: data, conf, all
    pub reset_type: ResetType,
}

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

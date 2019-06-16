//  Copyright (C) 2018  The Durs Project Developers.
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

//! Manage Durs core errors.

use crate::logger::InitLoggerError;
use dup_currency_params::db::CurrencyParamsDbError;
use durs_module::{ModuleStaticName, PlugModuleError};
use failure::Fail;

#[derive(Debug, Fail)]
/// Durs server error
pub enum DursCoreError {
    /// Error with configuration file
    #[fail(display = "Error with configuration file: {}", _0)]
    ConfFileError(durs_conf::DursConfFileError),
    /// Fail to read currency params DB
    #[fail(display = "Fail to read currency params DB: {}", _0)]
    FailReadCurrencyParamsDb(CurrencyParamsDbError),
    /// Fail to remove configuration file
    #[fail(display = "Fail to remove configuration file: {}", _0)]
    FailRemoveConfFile(std::io::Error),
    /// Fail to remove profile directory
    #[fail(display = "Fail to remove profile directory: {}", _0)]
    FailRemoveProfileDir(std::io::Error),
    /// Fail to remove datas directory
    #[fail(display = "Fail to remove datas directory: {}", _0)]
    FailRemoveDatasDir(std::io::Error),
    /// Fail to update configuration file
    #[fail(display = "Fail to update configuration file: {}", _0)]
    FailUpdateConf(std::io::Error),
    /// Fail to write keypairs file
    #[fail(display = "could not write keypairs file: {}", _0)]
    FailWriteKeypairsFile(std::io::Error),
    /// Error on initialization of the logger
    #[fail(display = "Error on initialization of the logger: {}", _0)]
    InitLoggerError(InitLoggerError),
    /// Plug module error
    #[fail(display = "Error on loading module '{}': {}", module_name, error)]
    PlugModuleError {
        /// Module name
        module_name: ModuleStaticName,
        /// Error details
        error: PlugModuleError,
    },
    /// Sync without source and without option local
    #[fail(display = "Please specify the url of a trusted node or use the --local option.")]
    SyncWithoutSource,
}

impl From<InitLoggerError> for DursCoreError {
    fn from(e: InitLoggerError) -> Self {
        DursCoreError::InitLoggerError(e)
    }
}

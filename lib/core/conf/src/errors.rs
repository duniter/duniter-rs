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

//! Dunitrust configuration errors

use failure::Fail;

/// Error with configuration file
#[derive(Debug, Fail)]
pub enum DursConfError {
    /// Env var error
    #[fail(display = "fail to parse configuration file: {}", _0)]
    EnvVarErr(DursConfEnvError),
    /// File error
    #[fail(display = "{}", _0)]
    FileErr(DursConfFileError),
}

/// Error with configuration file
#[derive(Debug, Fail)]
pub enum DursConfEnvError {
    /// Fail to parse conf version
    #[fail(display = "Fail to parse conf version : {}.", _0)]
    ConfVersionParseErr(std::num::ParseIntError),
    /// Envy error
    #[fail(display = "{}", _0)]
    EnvyErr(envy::Error),
    /// Unsupported version
    #[fail(
        display = "Version {} not supported. List of supported versions : {:?}.",
        found, expected
    )]
    UnsupportedVersion {
        /// List of supported versions
        expected: Vec<usize>,
        /// Version found
        found: usize,
    },
}

/// Error with configuration file
#[derive(Debug, Fail)]
pub enum DursConfFileError {
    /// Read error
    #[fail(display = "fail to read configuration file: {}", _0)]
    ReadError(std::io::Error),
    /// Parse error
    #[fail(display = "fail to parse configuration file: {}", _0)]
    ParseError(serde_json::Error),
    /// Write error
    #[fail(display = "fail to write configuration file: {}", _0)]
    WriteError(std::io::Error),
}

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

//! Dunitrust configuration constants

/// User datas folder.
pub static USER_DATAS_FOLDER: &str = "durs-dev";

/// Configuration filename.
pub static CONF_FILENAME: &str = "conf.json";

/// Keypairs filename.
pub static KEYPAIRS_FILENAME: &str = "keypairs.json";

/// If no currency is specified by the user, is the currency will be chosen by default.
pub static DEFAULT_CURRENCY: &str = "g1";

/// Default value for `default_sync_module` conf field.
pub static DEFAULT_DEFAULT_SYNC_MODULE: &str = "ws2p";

/// Modules datas folder.
pub static MODULES_DATAS_FOLDER: &str = "datas";

/// Prefix for dunitrust environment variables.
pub static DURS_ENV_PREFIX: &str = "DURS_";

/// Name of the environment variable that indicates the version of the configuration.
pub static DURS_CONF_VERSION: &str = "DURS_CONF_VERSION";

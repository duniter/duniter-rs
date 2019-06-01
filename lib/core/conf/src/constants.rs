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

//! Durs configuration constants

/// User datas folder
pub static USER_DATAS_FOLDER: &'static str = "durs-dev";

/// Configuration filename
pub static CONF_FILENAME: &'static str = "conf.json";

/// Keypairs filename
pub static KEYPAIRS_FILENAME: &'static str = "keypairs.json";

/// If no currency is specified by the user, is the currency will be chosen by default
pub static DEFAULT_CURRENCY: &'static str = "g1";

/// Default value for `default_sync_module` conf field
pub static DEFAULT_DEFAULT_SYNC_MODULE: &'static str = "ws2p";

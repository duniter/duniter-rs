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

//! Currency parameters constants.

/// Default currency name
pub const DEFAULT_CURRENCY: &str = "default_currency";
/// Default value for sig_renew_period parameter
pub static DEFAULT_SIG_RENEW_PERIOD: &'static u64 = &5_259_600;
/// Default value for ms_period parameter
pub static DEFAULT_MS_PERIOD: &'static u64 = &5_259_600;
/// Default value for tx_window parameter
pub static DEFAULT_TX_WINDOW: &'static u64 = &604_800;
/// Default maximum roolback length
pub static DEFAULT_FORK_WINDOW_SIZE: &'static usize = &200;

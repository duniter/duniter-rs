//  Copyright (C) 2018  The Dunitrust Project Developers.
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

//! WS2P V2+ Protocol constants

/// Connection negociation timeout
pub static WS2P_NEGOTIATION_TIMEOUT_IN_SECS: &'static u64 = &15;

/// Conection expiration timeout
pub static WS2P_EXPIRE_TIMEOUT_IN_SECS: &'static u64 = &120;

/// Interval between 2 messages from which it''s perhaps a spam (in milliseconds)
pub static WS2P_SPAM_INTERVAL_IN_MILLI_SECS: &'static u64 = &80;

/// Number of consecutive closed messages from which messages will be considered as spam.
pub static WS2P_SPAM_LIMIT: &'static usize = &6;

/// Rest time in a situation of proven spam
pub static WS2P_SPAM_SLEEP_TIME_IN_SEC: &'static u64 = &100;

/// Number of invalid messages tolerated
pub static WS2P_INVALID_MSGS_LIMIT: &'static usize = &5;

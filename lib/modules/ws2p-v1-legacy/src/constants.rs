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

//! WS2Pv1 constants

/// Module name
pub static MODULE_NAME: &'static str = "ws2p1";

/// API Name
pub static WS2P_API: &'static str = "WS2P";

/// Interval between to sequence of general actions
pub static WS2P_GENERAL_STATE_INTERVAL: &'static u64 = &15;

/// Connection wave interval at startup
pub static WS2P_OUTCOMING_INTERVAL_AT_STARTUP: &'static u64 = &75;

/// Interval of connection waves after the start-up phase
pub static WS2P_OUTCOMING_INTERVAL: &'static u64 = &300;

/// Default outgoing connection quota
pub static WS2P_DEFAULT_OUTCOMING_QUOTA: &'static usize = &10;

/// Maximum duration of a connection negotiation
pub static WS2P_NEGOTIATION_TIMEOUT: &'static u64 = &15;

/// Maximum waiting time for a response to a request
pub static WS2P_V1_REQUESTS_TIMEOUT_IN_SECS: &'static u64 = &30;

/// Maximum duration of inactivity of a connection (the connection will be closed after this delay)
pub static WS2P_EXPIRE_TIMEOUT: &'static u64 = &120;

/// Interval between 2 messages from which it''s perhaps a spam (in milliseconds)
pub static WS2P_SPAM_INTERVAL_IN_MILLI_SECS: &'static u64 = &80;

/// Number of consecutive closed messages from which messages will be considered as spam.
pub static WS2P_SPAM_LIMIT: &'static usize = &6;

/// Rest time in a situation of proven spam
pub static WS2P_SPAM_SLEEP_TIME_IN_SEC: &'static u64 = &100;

/// Duration between 2 endpoints saving
pub static DURATION_BETWEEN_2_ENDPOINTS_SAVING: &'static u64 = &180;

/// Duration between 2 requests from the pool of the wot data
pub static PENDING_IDENTITIES_REQUEST_INTERVAL: &'static u64 = &40;

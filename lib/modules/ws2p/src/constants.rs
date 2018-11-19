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

pub static WS2P_DEFAULT_OUTCOMING_QUOTA: &'static usize = &10;

/*pub static WS2P_OUTCOMING_INTERVAL_AT_STARTUP: &'static u64 = &75;
pub static WS2P_OUTCOMING_INTERVAL: &'static u64 = &300;*/
pub static WS2P_NEGOTIATION_TIMEOUT: &'static u64 = &15_000;
pub static WS2P_EXPIRE_TIMEOUT_IN_SECS: &'static u64 = &120;
pub static WS2P_RECV_SERVICE_FREQ_IN_MS: &'static u64 = &1_000;
pub static WS2P_SPAM_INTERVAL_IN_MILLI_SECS: &'static u64 = &80;
pub static WS2P_SPAM_LIMIT: &'static usize = &6;
pub static WS2P_SPAM_SLEEP_TIME_IN_SEC: &'static u64 = &100;
pub static WS2P_INVALID_MSGS_LIMIT: &'static usize = &5;
/*
pub static WS2P_REQUEST_TIMEOUT: &'static u64 = &30_000;
pub static DURATION_BEFORE_RECORDING_ENDPOINT: &'static u64 = &180;
pub static BLOCKS_REQUEST_INTERVAL: &'static u64 = &60;
pub static PENDING_IDENTITIES_REQUEST_INTERVAL: &'static u64 = &40;
*/

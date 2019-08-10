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

//! Define ws2p connections states.

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum WS2PConnectionState {
    NeverTry = 0,
    TryToOpenWS = 1,
    WSError = 2,
    TryToSendConnectMess = 3,
    Unreachable = 4,
    WaitingConnectMess = 5,
    NoResponse = 6,
    ConnectMessOk = 7,
    OkMessOkWaitingAckMess = 8,
    AckMessOk = 9,
    Denial = 10,
    Close = 11,
    Established = 12,
}

impl From<u32> for WS2PConnectionState {
    fn from(integer: u32) -> Self {
        match integer {
            1 | 2 => WS2PConnectionState::WSError,
            3 | 4 => WS2PConnectionState::Unreachable,
            5 | 6 => WS2PConnectionState::NoResponse,
            7 | 8 | 9 | 10 => WS2PConnectionState::Denial,
            11 | 12 => WS2PConnectionState::Close,
            _ => WS2PConnectionState::NeverTry,
        }
    }
}

impl WS2PConnectionState {
    pub fn from_u32(integer: u32, from_db: bool) -> Self {
        if from_db {
            WS2PConnectionState::from(integer)
        } else {
            match integer {
                1 => WS2PConnectionState::TryToOpenWS,
                2 => WS2PConnectionState::WSError,
                3 | 4 => WS2PConnectionState::Unreachable,
                5 | 6 => WS2PConnectionState::NoResponse,
                7 => WS2PConnectionState::ConnectMessOk,
                8 => WS2PConnectionState::OkMessOkWaitingAckMess,
                9 => WS2PConnectionState::AckMessOk,
                10 => WS2PConnectionState::Denial,
                11 => WS2PConnectionState::Close,
                12 => WS2PConnectionState::Established,
                _ => WS2PConnectionState::NeverTry,
            }
        }
    }
    pub fn to_u32(self) -> u32 {
        match self {
            WS2PConnectionState::NeverTry => 0,
            _ => 1,
        }
    }
}

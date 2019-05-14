// Copyright (C) 2018 The Durs Project Developers.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Process WS2P OK message.

use crate::controllers::handler::*;
use crate::controllers::*;
use ws::CloseCode;

/// Process WS2pv2 OK Message
pub fn process_ws2p_v2_ok_msg(handler: &mut Ws2pConnectionHandler) {
    debug!("Receive OK message !");

    match handler.conn_datas.state {
        WS2PConnectionState::ConnectMessOk => {
            handler.update_status(WS2PConnectionState::OkMsgOkWaitingAckMsg);
        }
        WS2PConnectionState::AckMsgOk => {
            handler.update_status(WS2PConnectionState::Established);
        }
        _ => {
            let _ = handler
                .ws
                .0
                .close_with_reason(CloseCode::Invalid, "Unexpected OK message !");
        }
    }
}

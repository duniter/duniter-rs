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

//! Sub-module process reception of OK message

use crate::connection_state::WS2PConnectionState;
use crate::controller::{
    WS2PController, WS2PControllerEvent, WS2PControllerProcessError, WebsocketActionOrder,
};
use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use log::error;
use unwrap::unwrap;

/// Process WS2P v2+ OK Message
pub fn process_ws2p_v2p_ok_msg<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    log::debug!("Receive OK message !");

    match controller.meta_datas.state {
        WS2PConnectionState::ConnectMessOk | WS2PConnectionState::SecretFlagsOkWaitingAckMsg => {
            controller.update_conn_state(WS2PConnectionState::OkMsgOkWaitingAckMsg)?;
            Ok(None)
        }
        WS2PConnectionState::AckMsgOk | WS2PConnectionState::SecretFlagsOk => {
            controller.meta_datas.state = WS2PConnectionState::Established;
            controller.send_event(WS2PControllerEvent::NewConnEstablished {
                conn_type: if controller.meta_datas.connect_type != WS2Pv2ConnectType::Incoming {
                    controller.meta_datas.connect_type
                } else {
                    unwrap!(controller.meta_datas.remote_connect_type)
                },
                remote_full_id: if let Some(ref remote_node) = controller.meta_datas.remote_node {
                    remote_node.remote_full_id
                } else {
                    fatal_error!("remote_node must be valued in process_ws2p_v2p_ok_msg() !")
                },
            })?;
            Ok(None)
        }
        _ => Ok(super::close_with_reason(
            "Unexpected OK message !",
            WS2PConnectionState::Denial,
        )),
    }
}

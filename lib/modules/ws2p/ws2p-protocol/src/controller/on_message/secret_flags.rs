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

//! Sub-module process reception of SECRET_FLAGS message

use crate::connection_state::WS2PConnectionState;
use crate::controller::{WS2PController, WS2PControllerProcessError, WebsocketActionOrder};
//use durs_common_tools::fatal_error;
use durs_module::ModuleMessage;
//use durs_ws2p_messages::v2::connect::WS2Pv2ConnectType;
use durs_ws2p_messages::v2::secret_flags::WS2Pv2SecretFlagsMsg;
//use log::error;
//use unwrap::unwrap;

pub fn process_ws2p_v2p_secret_flags_msg<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    secret_flags: &WS2Pv2SecretFlagsMsg,
) -> Result<Option<WebsocketActionOrder>, WS2PControllerProcessError> {
    // SECRET_FLAGS informations must never be logged in prod
    #[cfg(test)]
    log::debug!("Receive SECRET_FLAGS message !");

    match controller.meta_datas.state {
        WS2PConnectionState::ConnectMessOk => process(
            controller,
            secret_flags,
            WS2PConnectionState::SecretFlagsOkWaitingAckMsg,
        )
        .map(|_| None),
        WS2PConnectionState::AckMsgOk => {
            process(controller, secret_flags, WS2PConnectionState::SecretFlagsOk).map(|_| None)
        }
        _ => Ok(super::close_with_reason(
            "Unexpected SECRET_FLAGS message !",
            WS2PConnectionState::Denial,
        )),
    }
}

fn process<M: ModuleMessage>(
    controller: &mut WS2PController<M>,
    _secret_flags: &WS2Pv2SecretFlagsMsg,
    success_state: WS2PConnectionState,
) -> Result<(), WS2PControllerProcessError> {
    // TODO .. traitement des secrets flags
    controller.update_conn_state(success_state)
}

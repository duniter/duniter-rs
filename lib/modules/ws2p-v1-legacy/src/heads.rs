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

use crate::*;
use dubp_documents::Blockstamp;
use durs_network_documents::network_head_v2::*;

pub fn generate_my_head(
    network_keypair: &KeyPairEnum,
    node_id: NodeId,
    soft_name: &str,
    soft_version: &str,
    my_current_blockstamp: &Blockstamp,
    my_uid: Option<String>,
) -> NetworkHead {
    let message = NetworkHeadMessage::V2(NetworkHeadMessageV2 {
        api: String::from("WS2POCA"),
        version: 1,
        pubkey: network_keypair.public_key(),
        blockstamp: *my_current_blockstamp,
        node_uuid: node_id,
        software: String::from(soft_name),
        soft_version: String::from(soft_version),
        prefix: 1,
        free_member_room: None,
        free_mirror_room: None,
    });
    let message_v2 = NetworkHeadMessage::V2(NetworkHeadMessageV2 {
        api: String::from("WS2POCA"),
        version: 2,
        pubkey: network_keypair.public_key(),
        blockstamp: *my_current_blockstamp,
        node_uuid: node_id,
        software: String::from(soft_name),
        soft_version: String::from(soft_version),
        prefix: 1,
        free_member_room: Some(0),
        free_mirror_room: Some(0),
    });
    NetworkHead::V2(Box::new(NetworkHeadV2 {
        message: message.clone(),
        sig: network_keypair
            .private_key()
            .sign(message.to_string().as_bytes()),
        message_v2: message_v2.clone(),
        sig_v2: network_keypair
            .private_key()
            .sign(message_v2.to_string().as_bytes()),
        step: 0,
        uid: my_uid,
    }))
}

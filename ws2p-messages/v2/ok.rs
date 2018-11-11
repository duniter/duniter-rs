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

use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use std::num::NonZeroU16;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// WS2Pv2OkMsg
pub struct WS2Pv2OkMsg {
    /// If this field is zero, it means that the remote node does not want to reveal its prefix (the prefix being necessarily greater than or equal to 1).
    pub prefix: Option<NonZeroU16>,
    /// WS2Pv2SyncTarget
    pub sync_target: Option<WS2Pv2SyncTarget>,
}

impl Default for WS2Pv2OkMsg {
    fn default() -> Self {
        WS2Pv2OkMsg {
            prefix: None,
            sync_target: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// WS2Pv2SyncTarget
pub struct WS2Pv2SyncTarget {
    /// Indicates the current blockstamp of the message sender node. This blockstamp will be the target to reach for the node being synchronized.
    pub target_blockstamp: Blockstamp,
    /// Hash table of the last block of each chunk. We do not need the block numbers, we know them. Here the remote node sends the hashs of all these chunk, which correspond to the current hashs of all the blocks having a number in 250 module 249, in ascending order.
    pub chunks_hash: Vec<Hash>,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use dubp_documents::Blockstamp;
    use std::num::NonZeroU16;
    use tests::*;

    #[test]
    fn test_ws2p_message_ok() {
        let ok_msg = WS2Pv2OkMsg {
            prefix: NonZeroU16::new(1),
            sync_target: Some(WS2Pv2SyncTarget {
                target_blockstamp: Blockstamp::from_string(
                    "500-000011BABEEE1020B1F6B2627E2BC1C35BCD24375E114349634404D2C266D84F",
                )
                .unwrap(),
                chunks_hash: vec![
                    Hash::from_hex(
                        "000007722B243094269E548F600BD34D73449F7578C05BD370A6D301D20B5F10",
                    )
                    .unwrap(),
                    Hash::from_hex(
                        "0000095FD4C8EA96DE2844E3A4B62FD18761E9B4C13A74FAB716A4C81F438D91",
                    )
                    .unwrap(),
                ],
            }),
        };
        test_ws2p_message(WS2Pv0MessagePayload::Ok(ok_msg));
    }
}

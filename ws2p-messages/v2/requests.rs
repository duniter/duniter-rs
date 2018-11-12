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

use dubp_documents::{BlockId, Blockstamp};

/// WS2Pv2Request
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WS2Pv2Request {
    /// request unique identifier
    pub id: u32,
    /// request body
    pub body: WS2Pv2RequestBody,
}

impl WS2Pv2Request {
    /// Request size in binary format
    pub fn size_in_bytes(&self) -> usize {
        4 + self.body.size_in_bytes()
    }
}

/// WS2Pv2RequestBody
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum WS2Pv2RequestBody {
    /// Empty request
    None,
    /// Request current blockstamp
    Current,
    /// BLOCKS_HASHS : In case of fork, to quickly find the fork point, the node will request the hashes of the ForkWindowsSize of the local blockchains of the other nodes.
    /// It would be counterproductive to ask directly for the entire blocks, when you will only need them if you actually decide to stack the corresponding branch.
    /// param1: begin_block_id (u32)
    /// param2: blocks_count (u16)
    BlocksHashs(BlockId, u16),
    /// CHUNK: Request chunk of blocks.
    /// param1: begin_block_id (u32)
    /// param2: blocks_count (u16)
    Chunk(BlockId, u16),
    /// CHUNK_BY_HASH : During synchronization, chunk is requested by Chunkstamp (= Blockstamp of the last block of the chunk).
    ChunkByHash(Blockstamp),
    /// WOT_POOL : For network performance reasons, a Durs* node never shares its entire wot pool at once.
    /// It randomly selects folders_count folders among those having received at least min_cert certifications.
    /// It's the requesting node that sets the values of min_cert and folders_count according to its connection rate,
    /// its configuration and the rate of new folders it has obtained in these previous requests.
    /// param1: folders_count (u16)
    /// param2: min_cert (u8)
    WotPool(u16, u8),
}

impl WS2Pv2RequestBody {
    /// Request size in binary format
    pub fn size_in_bytes(&self) -> usize {
        match *self {
            WS2Pv2RequestBody::None | WS2Pv2RequestBody::Current => 1,
            WS2Pv2RequestBody::BlocksHashs(_, _) | WS2Pv2RequestBody::Chunk(_, _) => 7,
            WS2Pv2RequestBody::ChunkByHash(_) => 37,
            WS2Pv2RequestBody::WotPool(_, _) => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use dubp_documents::Blockstamp;
    use tests::*;

    #[test]
    fn test_ws2p_message_request() {
        let chunkstamp = Blockstamp::from_string(
            "499-000011BABEEE1020B1F6B2627E2BC1C35BCD24375E114349634404D2C266D84F",
        )
        .unwrap();
        let request = WS2Pv2Request {
            id: 27,
            body: WS2Pv2RequestBody::ChunkByHash(chunkstamp),
        };
        test_ws2p_message(WS2Pv0MessagePayload::Request(request));
    }
}

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

//! Sub-module managing the inter-modules requests sent.

use crate::*;
use durs_network::requests::OldNetworkRequest;

pub fn request_network_consensus(bc: &mut BlockchainModule) {
    let req = OldNetworkRequest::GetConsensus(ModuleReqFullId(
        BlockchainModule::name(),
        ModuleReqId(bc.pending_network_requests.len() as u32),
    ));
    let req_id = dunp::queries::request_network(
        bc,
        ModuleReqId(bc.pending_network_requests.len() as u32),
        &req,
    );
    bc.pending_network_requests.insert(req_id, req);
}

pub fn request_orphan_previous(bc: &mut BlockchainModule, orphan_blockstamp: Blockstamp) {
    let new_pending_network_requests =
        dunp::queries::request_orphan_previous(bc, orphan_blockstamp.id);
    for (new_req_id, new_req) in new_pending_network_requests {
        bc.pending_network_requests.insert(new_req_id, new_req);
    }
}

pub fn request_next_main_blocks(bc: &mut BlockchainModule) {
    // Choose frequency
    let frequency = if bc.consensus.id.0 == 0
        || bc.consensus.id.0 > bc.current_blockstamp.id.0 + *BLOCKS_DELAY_THRESHOLD
    {
        *REQUEST_MAIN_BLOCKS_HIGH_FREQUENCY_IN_SEC
    } else {
        *REQUEST_MAIN_BLOCKS_LOW_FREQUENCY_IN_SEC
    };

    // Apply frequency
    let now = SystemTime::now();
    if now
        .duration_since(bc.last_request_blocks)
        .expect("duration_since error")
        > Duration::from_secs(frequency)
    {
        bc.last_request_blocks = now;
        // Request next main blocks
        let to = match bc.consensus.id.0 {
            0 => (bc.current_blockstamp.id.0 + *MAX_BLOCKS_REQUEST),
            _ => bc.consensus.id.0,
        };
        let new_pending_network_requests = dunp::queries::request_blocks_to(bc, BlockNumber(to));
        for (new_req_id, new_req) in new_pending_network_requests {
            bc.pending_network_requests.insert(new_req_id, new_req);
        }
    }
}

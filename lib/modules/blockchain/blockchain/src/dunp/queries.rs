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

//! Sub-module that sends requests to the inter-node network layer.

use crate::*;
use durs_message::*;
use durs_module::ModuleReqId;
use durs_network::requests::OldNetworkRequest;

/// Send network request
pub fn request_network(
    bc: &BlockchainModule,
    req_id: ModuleReqId,
    request: &OldNetworkRequest,
) -> ModuleReqId {
    if bc
        .router_sender
        .send(RouterThreadMessage::ModuleMessage(DursMsg::Request {
            req_from: BlockchainModule::name(),
            req_to: ModuleRole::InterNodesNetwork,
            req_id,
            req_content: DursReqContent::OldNetworkRequest(*request),
        }))
        .is_err()
    {
        debug!("Fail to send OldNetworkRequest to router");
    }
    request.get_req_id()
}

/// Request chunk from network (chunk = group of blocks)
pub fn request_chunk(
    bc: &BlockchainModule,
    req_id: ModuleReqId,
    from: u32,
) -> (ModuleReqId, OldNetworkRequest) {
    let req = OldNetworkRequest::GetBlocks(
        ModuleReqFullId(BlockchainModule::name(), req_id),
        *CHUNK_SIZE,
        from,
    );
    (request_network(bc, req_id, &req), req)
}
/// Requests blocks from current to `to`
pub fn request_blocks_to(
    bc: &BlockchainModule,
    to: BlockNumber,
) -> HashMap<ModuleReqId, OldNetworkRequest> {
    let from = if bc.current_blockstamp == Blockstamp::default() {
        0
    } else {
        bc.current_blockstamp.id.0 + 1
    };
    info!(
        "BlockchainModule : request_blocks_to({}-{})",
        bc.current_blockstamp.id.0 + 1,
        to
    );
    if bc.current_blockstamp.id < to {
        let real_to = if (to.0 - bc.current_blockstamp.id.0) > *MAX_BLOCKS_REQUEST {
            bc.current_blockstamp.id.0 + *MAX_BLOCKS_REQUEST
        } else {
            to.0
        };
        request_blocks_from_to(bc, BlockNumber(from), BlockNumber(real_to))
    } else {
        HashMap::with_capacity(0)
    }
}

/// Requets previous blocks from specific orphan block
#[inline]
pub fn request_orphan_previous(
    _bc: &BlockchainModule,
    _orphan_block_number: BlockNumber,
) -> HashMap<ModuleReqId, OldNetworkRequest> {
    /*if orphan_block_number.0
        > bc.current_blockstamp.id.0 - *durs_blockchain_dal::constants::FORK_WINDOW_SIZE as u32
        && orphan_block_number.0 <= bc.current_blockstamp.id.0 + *CHUNK_SIZE
    {
        request_blocks_from_to(
            bc,
            orphan_block_number.0 - *CHUNK_SIZE + 1,
            orphan_block_number.0,
        )
    } else {*/
    HashMap::with_capacity(0)
}

/// Requests blocks from `from` to `to`
pub fn request_blocks_from_to(
    bc: &BlockchainModule,
    from: BlockNumber,
    to: BlockNumber,
) -> HashMap<ModuleReqId, OldNetworkRequest> {
    info!("BlockchainModule : request_blocks_from_to({}-{})", from, to);
    let mut from = from.0;
    let to = to.0;
    let mut requests_ids = HashMap::new();
    while from <= to {
        let mut req_id = ModuleReqId(0);
        while bc.pending_network_requests.contains_key(&req_id)
            || requests_ids.contains_key(&req_id)
        {
            req_id = ModuleReqId(req_id.0 + 1);
        }
        let (req_id, req) = request_chunk(bc, req_id, from);
        requests_ids.insert(req_id, req);
        from += *CHUNK_SIZE;
    }
    requests_ids
}

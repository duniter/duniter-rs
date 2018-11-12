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

//! Defined network requests.

use documents::*;
use duniter_module::*;
use *;

#[derive(Debug, Copy, Clone)]
/// Type containing a request addressed to the network module
pub enum OldNetworkRequest {
    /// Get a current block of a specific node
    GetCurrent(ModuleReqFullId, NodeFullId),
    //GetBlock(NodeFullId, u64),
    /// Get a blocks chunk from specified node
    GetBlocks(ModuleReqFullId, NodeFullId, u32, u32),
    /// Get pending wot documents from specified node
    GetRequirementsPending(ModuleReqFullId, NodeFullId, u32),
    /// Obtain the current network consensus
    GetConsensus(ModuleReqFullId),
    /// Getting the heads cache
    GetHeadsCache(ModuleReqFullId),
    /// Get a list of known endpoints
    GetEndpoints(ModuleReqFullId),
}

impl OldNetworkRequest {
    /// Get request full identitifier
    pub fn get_req_full_id(&self) -> ModuleReqFullId {
        match *self {
            OldNetworkRequest::GetCurrent(ref req_id, _)
            | OldNetworkRequest::GetBlocks(ref req_id, _, _, _)
            | OldNetworkRequest::GetRequirementsPending(ref req_id, _, _)
            | OldNetworkRequest::GetConsensus(ref req_id)
            | OldNetworkRequest::GetHeadsCache(ref req_id)
            | OldNetworkRequest::GetEndpoints(ref req_id) => *req_id,
        }
    }
    /// Get request identitifier
    pub fn get_req_id(&self) -> ModuleReqId {
        self.get_req_full_id().1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Type returned when the network module does not get a satisfying answer to a request
pub enum OldNetworkRequestError {
    /// Receiving an invalid format response
    WrongFormat(),
    /// Unknow error
    UnknowError(),
    /// No response received
    NoResponse(),
    /// Unable to reach the target node
    ReceiverUnreachable(),
}

#[derive(Debug, Clone)]
/// Type containing the response to a network request
pub enum NetworkResponse {
    /// CurrentBlock
    CurrentBlock(ModuleReqFullId, NodeFullId, Box<NetworkBlock>),
    /// Block
    Block(ModuleReqFullId, NodeFullId, Box<NetworkBlock>),
    /// Chunk
    Chunk(ModuleReqFullId, NodeFullId, Vec<Box<NetworkBlock>>),
    /// PendingDocuments
    PendingDocuments(ModuleReqFullId, Vec<BlockchainDocument>),
    /// Consensus
    Consensus(ModuleReqFullId, Result<Blockstamp, NetworkConsensusError>),
    /// HeadsCache
    HeadsCache(ModuleReqFullId, Box<NetworkHead>),
}

impl NetworkResponse {
    /// Get request full identifier
    pub fn get_req_full_id(&self) -> ModuleReqFullId {
        match *self {
            NetworkResponse::CurrentBlock(ref req_id, _, _)
            | NetworkResponse::Block(ref req_id, _, _)
            | NetworkResponse::Chunk(ref req_id, _, _)
            | NetworkResponse::PendingDocuments(ref req_id, _)
            | NetworkResponse::Consensus(ref req_id, _)
            | NetworkResponse::HeadsCache(ref req_id, _) => *req_id,
        }
    }
    /// Get request identifier
    pub fn get_req_id(&self) -> ModuleReqId {
        self.get_req_full_id().1
    }
}

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
use dubp_documents::BlockNumber;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_blockchain_dal::filters::identities::IdentitiesFilter;
use durs_network::requests::OldNetworkRequest;

#[derive(Debug, Clone)]
/// Modules request content
pub enum DursReqContent {
    /// Request to the old network module
    OldNetworkRequest(OldNetworkRequest),
    /// Network request (Not yet implemented)
    NetworkRequest(),
    /// Blockchain datas request
    BlockchainRequest(BlockchainRequest),
    /// Mem pool datas request
    MemPoolRequest(MemPoolRequest),
    /// Request to the pow module
    ProverRequest(BlockNumber, Hash),
    /// Arbitrary datas
    ArbitraryDatas(ArbitraryDatas),
}

#[derive(Debug, Clone, PartialEq)]
/// Inter-module Blockchain request for blockchain data
pub enum BlockchainRequest {
    /// Current blockstamp
    CurrentBlockstamp(),
    /// Current block
    CurrentBlock,
    /// Block by number
    BlockByNumber {
        /// Block number
        block_number: BlockNumber,
    },
    /// Chunk (block pack)
    Chunk {
        /// First block number
        first_block_number: BlockNumber,
        /// Number of blocks
        count: u32,
    },
    /// Usernames corresponding to the public keys in parameter
    UIDs(Vec<PubKey>),
    /// Get identities
    GetIdentities(IdentitiesFilter),
}

#[derive(Debug, Copy, Clone)]
/// Inter-module request for mem pool data
pub enum MemPoolRequest {
    /// All pending identities with their pending certifications
    AllPendingIdentities(usize),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(usize),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(PubKey),
}

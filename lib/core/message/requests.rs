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

use crate::*;
use dubp_documents::BlockId;
use duniter_network::requests::OldNetworkRequest;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;

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
    ProverRequest(BlockId, Hash),
    /// Arbitrary datas
    ArbitraryDatas(ArbitraryDatas),
}

#[derive(Debug, Clone, PartialEq)]
/// Inter-module Blockchain request for blockchain data
pub enum BlockchainRequest {
    /// Current blockstamp
    CurrentBlockstamp(),
    /// Current block
    CurrentBlock(),
    /// Block by number
    BlockByNumber(u64),
    /// Chunk (block pack)
    Chunk(u64, usize),
    /// Usernames corresponding to the public keys in parameter
    UIDs(Vec<PubKey>),
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

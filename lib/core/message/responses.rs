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

use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::certification::CertificationDocument;
use dubp_documents::documents::identity::IdentityDocument;
use dubp_documents::documents::membership::MembershipDocument;
use dubp_documents::documents::revocation::RevocationDocument;
use dubp_documents::BlockId;
use dubp_documents::Blockstamp;
use duniter_module::ModuleReqId;
use duniter_network::requests::NetworkResponse;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use std::collections::HashMap;

/// Durs request response message
#[derive(Debug, Clone)]
pub enum DursResContent {
    /// BlockchainResponse
    BlockchainResponse(BlockchainResponse),
    /// MemPoolResponse
    MemPoolResponse(MemPoolResponse),
    /// Response of OldNetworkRequest
    NetworkResponse(NetworkResponse),
    /// Pow module response
    ProverResponse(BlockId, Sig, u64),
}

#[derive(Debug, Clone)]
/// Pending identity datas
pub struct PendingIdtyDatas {
    /// Identity document
    pub idty: IdentityDocument,
    /// Membership document
    pub memberships: Vec<MembershipDocument>,
    /// Number of certifications received
    pub certs_count: usize,
    /// Certifications documents
    pub certs: Vec<CertificationDocument>,
    /// Revocation document (None if identity has not been revoked)
    pub revocation: Option<RevocationDocument>,
}

#[derive(Debug, Clone)]
/// Response to a BlockchainReqBlockchain request
pub enum BlockchainResponse {
    /// Current blockstamp
    CurrentBlockstamp(ModuleReqId, Blockstamp),
    /// Current block
    CurrentBlock(ModuleReqId, Box<BlockDocument>, Blockstamp),
    /// Block by number
    BlockByNumber(ModuleReqId, Box<BlockDocument>),
    /// Chunk (block pack)
    Chunk(ModuleReqId, Vec<BlockDocument>),
    /// Usernames corresponding to the public keys in parameter
    UIDs(ModuleReqId, HashMap<PubKey, Option<String>>),
    /// Identities
    Identities(ModuleReqId, Vec<IdentityDocument>),
}

#[derive(Debug, Clone)]
/// Response to a MemPoolRequest request
pub enum MemPoolResponse {
    /// All pending identities with their pending certifications
    AllPendingIdentities(ModuleReqId, HashMap<Hash, PendingIdtyDatas>),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(ModuleReqId, HashMap<Hash, PendingIdtyDatas>),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(ModuleReqId, Box<PendingIdtyDatas>),
}

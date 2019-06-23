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
use dubp_documents::documents::revocation::RevocationDocumentV10;
use dubp_documents::BlockNumber;
use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_module::ModuleReqId;
use durs_network::requests::NetworkResponse;
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
    ProverResponse(BlockNumber, Sig, u64),
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
    pub revocation: Option<RevocationDocumentV10>,
}

#[derive(Debug, Clone)]
/// Response to a BlockchainReqBlockchain request
pub enum BlockchainResponse {
    /// Current blockstamp
    CurrentBlockstamp(Blockstamp),
    /// Current block
    CurrentBlock(Box<BlockDocument>, Blockstamp),
    /// Block by number
    BlockByNumber(Box<BlockDocument>),
    /// Chunk (block pack)
    Chunk(Vec<BlockDocument>),
    /// Usernames corresponding to the public keys in parameter
    UIDs(HashMap<PubKey, Option<String>>),
    /// Identities
    Identities(Vec<IdentityDocument>),
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

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

extern crate duniter_module;
extern crate serde;

use self::duniter_module::ModuleReqId;
use dubp_documents::v10::block::BlockDocument;
use dubp_documents::v10::certification::CertificationDocument;
use dubp_documents::v10::identity::IdentityDocument;
use dubp_documents::v10::membership::MembershipDocument;
use dubp_documents::v10::revocation::RevocationDocument;
use dubp_documents::Blockstamp;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
/// Inter-module DAL request for pool data
pub enum DALReqPendings {
    /// All pending identities with their pending certifications
    AllPendingIdentities(usize),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(usize),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(PubKey),
}

#[derive(Debug, Clone, PartialEq)]
/// Inter-module DAL request for blockchain data
pub enum DALReqBlockchain {
    /// Current block
    CurrentBlock(),
    /// Block by number
    BlockByNumber(u64),
    /// Chunk (block pack)
    Chunk(u64, usize),
    /// Usernames corresponding to the public keys in parameter
    UIDs(Vec<PubKey>),
}

#[derive(Debug, Clone)]
/// Inter-module DAL request
pub enum DALRequest {
    /// Inter-module DAL request for blockchain data
    BlockchainRequest(DALReqBlockchain),
    /// Inter-module DAL request for pool data
    PendingsRequest(DALReqPendings),
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
/// Response to a DALReqPendings request
pub enum DALResPendings {
    /// All pending identities with their pending certifications
    AllPendingIdentities(ModuleReqId, HashMap<Hash, PendingIdtyDatas>),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(ModuleReqId, HashMap<Hash, PendingIdtyDatas>),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(ModuleReqId, Box<PendingIdtyDatas>),
}

#[derive(Debug, Clone)]
/// Response to a DALReqBlockchain request
pub enum DALResBlockchain {
    /// Current block
    CurrentBlock(ModuleReqId, Box<BlockDocument>, Blockstamp),
    /// Block by number
    BlockByNumber(ModuleReqId, Box<BlockDocument>),
    /// Chunk (block pack)
    Chunk(ModuleReqId, Vec<BlockDocument>),
    /// Usernames corresponding to the public keys in parameter
    UIDs(ModuleReqId, HashMap<PubKey, Option<String>>),
}

#[derive(Debug, Clone)]
/// Response to a DAL request
pub enum DALResponse {
    /// Response to a DALReqBlockchain request
    Blockchain(Box<DALResBlockchain>),
    /// Response to a DALReqPendings request
    Pendings(Box<DALResPendings>),
}

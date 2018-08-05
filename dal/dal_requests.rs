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

use self::duniter_module::ModuleReqFullId;
use duniter_crypto::hashs::Hash;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::{
    BlockDocument, CertificationDocument, IdentityDocument, MembershipDocument, RevocationDocument,
};
use duniter_documents::Blockstamp;
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// Inter-module DAL request for pool data
pub enum DALReqPendings {
    /// All pending identities with their pending certifications
    AllPendingIdentities(ModuleReqFullId, usize),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(ModuleReqFullId, usize),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(ModuleReqFullId, PubKey),
}

#[derive(Debug, Clone, PartialEq)]
/// Inter-module DAL request for blockchain data
pub enum DALReqBlockchain {
    /// Current block
    CurrentBlock(ModuleReqFullId),
    /// Block by number
    BlockByNumber(ModuleReqFullId, u64),
    /// Chunk (block pack)
    Chunk(ModuleReqFullId, u64, usize),
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
    AllPendingIdentities(HashMap<Hash, PendingIdtyDatas>),
    /// All pending identities without their pending certifications
    AllPendingIdentitiesWithoutCerts(HashMap<Hash, PendingIdtyDatas>),
    /// All pending datas for given pubkey
    PendingWotDatasForPubkey(Box<PendingIdtyDatas>),
}

#[derive(Debug, Clone)]
/// Response to a DALReqBlockchain request
pub enum DALResBlockchain {
    /// Current block
    CurrentBlock(ModuleReqFullId, Box<BlockDocument>, Blockstamp),
    /// Block by number
    BlockByNumber(ModuleReqFullId, Box<BlockDocument>),
    /// Chunk (block pack)
    Chunk(ModuleReqFullId, Vec<BlockDocument>),
    /// Usernames corresponding to the public keys in parameter
    UIDs(HashMap<PubKey, Option<String>>),
}

#[derive(Debug, Clone)]
/// Response to a DAL request
pub enum DALResponse {
    /// Response to a DALReqBlockchain request
    Blockchain(Box<DALResBlockchain>),
    /// Response to a DALReqPendings request
    Pendings(ModuleReqFullId, Box<DALResPendings>),
}

extern crate duniter_module;
extern crate serde;

use self::duniter_module::ModuleReqFullId;
use duniter_crypto::keys::*;
use duniter_documents::blockchain::v10::documents::{
    BlockDocument, CertificationDocument, IdentityDocument, MembershipDocument, RevocationDocument,
};
use duniter_documents::{Blockstamp, Hash};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum DALReqPendings {
    AllPendingIdentyties(ModuleReqFullId, usize),
    AllPendingIdentytiesWithoutCerts(ModuleReqFullId, usize),
    PendingWotDatasForPubkey(ModuleReqFullId, PubKey),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DALReqBlockchain {
    CurrentBlock(ModuleReqFullId),
    BlockByNumber(ModuleReqFullId, u64),
    Chunk(ModuleReqFullId, u64, usize),
    UIDs(Vec<PubKey>),
}

#[derive(Debug, Clone)]
pub enum DALRequest {
    BlockchainRequest(DALReqBlockchain),
    PendingsRequest(DALReqPendings),
}

#[derive(Debug, Clone)]
pub struct PendingIdtyDatas {
    pub idty: IdentityDocument,
    pub memberships: Vec<MembershipDocument>,
    pub certs_count: usize,
    pub certs: Vec<CertificationDocument>,
    pub revocation: Option<RevocationDocument>,
}

#[derive(Debug, Clone)]
pub enum DALResPendings {
    AllPendingIdentyties(HashMap<Hash, PendingIdtyDatas>),
    AllPendingIdentytiesWithoutCerts(HashMap<Hash, PendingIdtyDatas>),
    PendingWotDatasForPubkey(Vec<PendingIdtyDatas>),
}

#[derive(Debug, Clone)]
pub enum DALResBlockchain {
    CurrentBlock(ModuleReqFullId, Box<BlockDocument>, Blockstamp),
    BlockByNumber(ModuleReqFullId, Box<BlockDocument>),
    Chunk(ModuleReqFullId, Vec<BlockDocument>),
    UIDs(HashMap<PubKey, Option<String>>),
}

#[derive(Debug, Clone)]
pub enum DALResponse {
    Blockchain(Box<DALResBlockchain>),
    Pendings(ModuleReqFullId, DALResPendings),
}

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

//! Module managing the Duniter blockchain.

#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "cargo-clippy", allow(unused_collect))]
#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;

extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate duniter_wotb;
extern crate serde;
extern crate serde_json;
extern crate sqlite;

mod apply_valid_block;
mod check_and_apply_block;
mod dbex;
mod sync;
mod ts_parsers;

use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use apply_valid_block::*;
use check_and_apply_block::*;
pub use dbex::{DBExQuery, DBExTxQuery, DBExWotQuery};
use duniter_crypto::keys::*;
use duniter_dal::block::DALBlock;
use duniter_dal::currency_params::CurrencyParameters;
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALReqBlockchain, DALRequest, DALResBlockchain, DALResponse};
use duniter_dal::identity::DALIdentity;
use duniter_dal::writers::requests::BlocksDBsWriteQuery;
use duniter_dal::*;
use duniter_documents::blockchain::v10::documents::{BlockDocument, V10Document};
use duniter_documents::blockchain::{BlockchainProtocol, Document};
use duniter_documents::*;
use duniter_message::DuniterMessage;
use duniter_module::*;
use duniter_network::{
    NetworkBlock, NetworkDocument, NetworkEvent, NetworkRequest, NetworkResponse, NodeFullId,
};
use duniter_wotb::data::rusty::RustyWebOfTrust;
use duniter_wotb::operations::file::BinaryFileFormater;
use duniter_wotb::{NodeId, WebOfTrust};

/// The blocks are requested by packet groups. This constant sets the block packet size.
pub static CHUNK_SIZE: &'static u32 = &50;
/// Necessary to instantiate the wot object before knowing the currency parameters
pub static INFINITE_SIG_STOCK: &'static usize = &4_000_000_000;
/// The blocks are requested by packet groups. This constant sets the number of packets per group.
pub static MAX_BLOCKS_REQUEST: &'static u32 = &500;
/// There can be several implementations of the wot file backup, this constant fixes the implementation used by the blockchain module.
pub static WOT_FILE_FORMATER: BinaryFileFormater = BinaryFileFormater {};

/// Blockchain Module
#[derive(Debug)]
pub struct BlockchainModule {
    /// Subscribers
    pub followers: Vec<mpsc::Sender<DuniterMessage>>,
    /// Name of the user datas profile
    pub conf_profile: String,
    /// Currency
    pub currency: Currency,
    // Currency parameters
    currency_params: CurrencyParameters,
    /// Wots Databases
    pub wot_databases: WotsV10DBs,
    /// Blocks Databases
    pub blocks_databases: BlocksV10DBs,
    /// Currency databases
    currency_databases: CurrencyV10DBs,
    /// The block under construction
    pub pending_block: Option<Box<BlockDocument>>,
    /// Current state of all forks
    pub forks_states: Vec<ForkStatus>,
}

#[derive(Debug, Clone)]
/// Block
pub enum Block<'a> {
    /// Block coming from Network
    NetworkBlock(&'a NetworkBlock),
    /// Block coming from local database
    LocalBlock(&'a BlockDocument),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// When synchronizing the blockchain, checking all rules at each block really takes a long time.
/// The user is therefore offered a fast synchronization that checks only what is strictly necessary for indexing the data.
pub enum SyncVerificationLevel {
    /// Fast sync, checks only what is strictly necessary for indexing the data.
    FastSync(),
    /// Cautious sync, checking all protocol rules (really takes a long time).
    Cautious(),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Error returned by function complete_network_block()
pub enum CompletedBlockError {
    /// Invalid block inner hash
    InvalidInnerHash(),
    /// Invalid block hash
    InvalidHash(),
    /// Invalid block version
    InvalidVersion(),
}

impl BlockchainModule {
    /// Return module identifier
    pub fn id() -> ModuleId {
        ModuleId::Str("blockchain")
    }
    /// Loading blockchain configuration
    pub fn load_blockchain_conf(
        conf: &DuniterConf,
        _keys: RequiredKeysContent,
    ) -> BlockchainModule {
        // Get db path
        let db_path =
            duniter_conf::get_blockchain_db_path(conf.profile().as_str(), &conf.currency());

        // Open databases
        let blocks_databases = BlocksV10DBs::open(&db_path, false);
        let wot_databases = WotsV10DBs::open(&db_path, false);
        let currency_databases = CurrencyV10DBs::open(&db_path, false);

        // Get current blockstamp
        let current_blockstamp = duniter_dal::block::get_current_blockstamp(&blocks_databases)
            .expect("Fatal error : fail to read Blockchain DB !");

        // Get currency parameters
        let currency_params = duniter_dal::currency_params::get_currency_params(
            &blocks_databases.blockchain_db,
        ).expect("Fatal error : fail to read Blockchain DB !")
            .unwrap_or_default();

        // Get forks states
        let forks_states = if let Some(current_blockstamp) = current_blockstamp {
            duniter_dal::block::get_forks(&blocks_databases.forks_db, current_blockstamp)
                .expect("Fatal error : fail to read Forks DB !")
        } else {
            vec![]
        };

        // Instanciate BlockchainModule
        BlockchainModule {
            followers: Vec::new(),
            conf_profile: conf.profile(),
            currency: conf.currency(),
            currency_params,
            blocks_databases,
            wot_databases,
            currency_databases,
            pending_block: None,
            forks_states,
        }
    }
    /// Databases explorer
    pub fn dbex(conf: &DuniterConf, req: &DBExQuery) {
        dbex::dbex(conf, req);
    }
    /// Synchronize blockchain from a duniter-ts database
    pub fn sync_ts(conf: &DuniterConf, ts_profile: &str, cautious: bool) {
        // get databases path
        let db_path = duniter_conf::get_blockchain_db_path(&conf.profile(), &conf.currency());
        // Open blocks dbs
        let blocks_dbs = BlocksV10DBs::open(&db_path, false);
        // Get local current blockstamp
        debug!("Get local current blockstamp...");
        let current_blockstamp: Blockstamp = duniter_dal::block::get_current_blockstamp(
            &blocks_dbs,
        ).expect("ForksV10DB : RustBreakError !")
            .unwrap_or_default();
        debug!("Success to get local current blockstamp.");
        // get db_ts_path
        let mut db_ts_path = match env::home_dir() {
            Some(path) => path,
            None => panic!("Impossible to get your home dir!"),
        };
        db_ts_path.push(".config/duniter/");
        db_ts_path.push(ts_profile);
        db_ts_path.push("duniter.db");
        if !db_ts_path.as_path().exists() {
            panic!("Fatal error : duniter-ts database don't exist !");
        }
        sync::sync_ts(conf, &current_blockstamp, db_ts_path, cautious);
    }
    /// Request chunk from network (chunk = group of blocks)
    fn request_chunk(&self, req_id: &ModuleReqId, from: u32) -> (ModuleReqId, NetworkRequest) {
        let req = NetworkRequest::GetBlocks(
            ModuleReqFullId(BlockchainModule::id(), *req_id),
            NodeFullId::default(),
            *CHUNK_SIZE,
            from,
        );
        (self.request_network(req), req)
    }
    /// Requests blocks from current to `to`
    fn request_blocks_to(
        &self,
        pending_network_requests: &HashMap<ModuleReqId, NetworkRequest>,
        current_blockstamp: &Blockstamp,
        to: BlockId,
    ) -> HashMap<ModuleReqId, NetworkRequest> {
        let mut from = if *current_blockstamp == Blockstamp::default() {
            0
        } else {
            current_blockstamp.id.0 + 1
        };
        info!(
            "BlockchainModule : request_blocks_to({}-{})",
            current_blockstamp.id.0 + 1,
            to
        );
        let mut requests_ids = HashMap::new();
        if current_blockstamp.id < to {
            let real_to = if (to.0 - current_blockstamp.id.0) > *MAX_BLOCKS_REQUEST {
                current_blockstamp.id.0 + *MAX_BLOCKS_REQUEST
            } else {
                to.0
            };
            while from <= real_to {
                let mut req_id = ModuleReqId(0);
                while pending_network_requests.contains_key(&req_id)
                    || requests_ids.contains_key(&req_id)
                {
                    req_id = ModuleReqId(req_id.0 + 1);
                }
                let (req_id, req) = self.request_chunk(&req_id, from);
                requests_ids.insert(req_id, req);
                from += *CHUNK_SIZE;
            }
        }
        requests_ids
    }
    /// Send network request
    fn request_network(&self, request: NetworkRequest) -> ModuleReqId {
        for follower in &self.followers {
            if follower
                .send(DuniterMessage::NetworkRequest(request))
                .is_err()
            {
                debug!("BlockchainModule : one follower is unreachable !");
            }
        }
        request.get_req_id()
    }
    /// Send blockchain event
    fn send_event(&self, event: &DALEvent) {
        for follower in &self.followers {
            if follower
                .send(DuniterMessage::DALEvent(event.clone()))
                .is_err()
            {
                // Handle error
            }
        }
    }
    fn send_req_response(&self, response: &DALResponse) {
        for follower in &self.followers {
            if follower
                .send(DuniterMessage::DALResponse(Box::new(response.clone())))
                .is_err()
            {
                // Handle error
            }
        }
    }
    fn receive_network_documents<W: WebOfTrust + Sync>(
        &mut self,
        network_documents: &[NetworkDocument],
        current_blockstamp: &Blockstamp,
        wotb_index: &mut HashMap<PubKey, NodeId>,
        wot: &mut W,
    ) -> Blockstamp {
        let mut blockchain_documents = Vec::new();
        let mut current_blockstamp = *current_blockstamp;
        let mut save_blocks_dbs = false;
        let mut save_wots_dbs = false;
        let mut save_currency_dbs = false;
        for network_document in network_documents {
            match *network_document {
                NetworkDocument::Block(ref network_block) => {
                    match check_and_apply_block(
                        &self.blocks_databases,
                        &self.wot_databases.certs_db,
                        &Block::NetworkBlock(network_block),
                        &current_blockstamp,
                        wotb_index,
                        wot,
                        &self.forks_states,
                    ) {
                        Ok(ValidBlockApplyReqs(block_req, wot_dbs_reqs, currency_dbs_reqs)) => {
                            let block_doc = network_block.uncompleted_block_doc().clone();
                            // Apply wot dbs requests
                            wot_dbs_reqs
                                .iter()
                                .map(|req| {
                                    req.apply(&self.wot_databases, &self.currency_params)
                                            .expect(
                                            "Fatal error : fail to apply WotsDBsWriteQuery : DALError !",
                                        )
                                })
                                .collect::<()>();
                            // Apply currency dbs requests
                            currency_dbs_reqs
                                .iter()
                                .map(|req| {
                                    req.apply(&self.currency_databases).expect(
                                            "Fatal error : fail to apply CurrencyDBsWriteQuery : DALError !",
                                        )
                                })
                                .collect::<()>();
                            // Write block
                            block_req.apply(&self.blocks_databases, false).expect(
                                "Fatal error : fail to write block in BlocksDBs : DALError !",
                            );
                            if let BlocksDBsWriteQuery::WriteBlock(_, _, _, block_hash) = block_req
                            {
                                info!("StackUpValidBlock({})", block_doc.number.0);
                                self.send_event(&DALEvent::StackUpValidBlock(
                                    Box::new(block_doc.clone()),
                                    Blockstamp {
                                        id: block_doc.number,
                                        hash: block_hash,
                                    },
                                ));
                            }
                            current_blockstamp = network_block.blockstamp();
                            // Update forks states
                            self.forks_states = duniter_dal::block::get_forks(
                                &self.blocks_databases.forks_db,
                                current_blockstamp,
                            ).expect("get_forks() : DALError");
                            save_blocks_dbs = true;
                            if !wot_dbs_reqs.is_empty() {
                                save_wots_dbs = true;
                            }
                            if !block_doc.transactions.is_empty()
                                || (block_doc.dividend.is_some()
                                    && block_doc.dividend.expect("safe unwrap") > 0)
                            {
                                save_currency_dbs = true;
                            }
                        }
                        Err(_) => {
                            warn!(
                                "RefusedBlock({})",
                                network_block.uncompleted_block_doc().number.0
                            );
                            self.send_event(&DALEvent::RefusedPendingDoc(BlockchainProtocol::V10(
                                Box::new(V10Document::Block(Box::new(
                                    network_block.uncompleted_block_doc().clone(),
                                ))),
                            )));
                        }
                    }
                }
                NetworkDocument::Identity(ref doc) => blockchain_documents.push(
                    BlockchainProtocol::V10(Box::new(V10Document::Identity(doc.deref().clone()))),
                ),
                NetworkDocument::Membership(ref doc) => blockchain_documents.push(
                    BlockchainProtocol::V10(Box::new(V10Document::Membership(doc.deref().clone()))),
                ),
                NetworkDocument::Certification(ref doc) => {
                    blockchain_documents.push(BlockchainProtocol::V10(Box::new(
                        V10Document::Certification(Box::new(doc.deref().clone())),
                    )))
                }
                NetworkDocument::Revocation(ref doc) => {
                    blockchain_documents.push(BlockchainProtocol::V10(Box::new(
                        V10Document::Revocation(Box::new(doc.deref().clone())),
                    )))
                }
                NetworkDocument::Transaction(ref doc) => {
                    blockchain_documents.push(BlockchainProtocol::V10(Box::new(
                        V10Document::Transaction(Box::new(doc.deref().clone())),
                    )))
                }
            }
        }
        if !blockchain_documents.is_empty() {
            self.receive_documents(&blockchain_documents);
        }
        // Save databases
        if save_blocks_dbs {
            self.blocks_databases.save_dbs();
        }
        if save_wots_dbs {
            self.wot_databases.save_dbs();
        }
        if save_currency_dbs {
            self.currency_databases.save_dbs(true, true);
        }
        current_blockstamp
    }
    fn receive_documents(&self, documents: &[BlockchainProtocol]) {
        debug!("BlockchainModule : receive_documents()");
        for document in documents {
            trace!("BlockchainModule : Treat one document.");
            match *document {
                BlockchainProtocol::V10(ref doc_v10) => match doc_v10.deref() {
                    _ => {}
                },
                _ => self.send_event(&DALEvent::RefusedPendingDoc(document.clone())),
            }
        }
    }
    fn receive_blocks<W: WebOfTrust + Sync>(
        &mut self,
        blocks_in_box: &[Box<NetworkBlock>],
        current_blockstamp: &Blockstamp,
        wotb_index: &mut HashMap<PubKey, NodeId>,
        wot: &mut W,
    ) -> Blockstamp {
        debug!("BlockchainModule : receive_blocks()");
        let blocks: Vec<&NetworkBlock> = blocks_in_box.into_iter().map(|b| b.deref()).collect();
        let mut current_blockstamp = *current_blockstamp;
        let mut save_blocks_dbs = false;
        let mut save_wots_dbs = false;
        let mut save_currency_dbs = false;
        for block in blocks {
            if let Ok(ValidBlockApplyReqs(bc_db_query, wot_dbs_queries, tx_dbs_queries)) =
                check_and_apply_block::<W>(
                    &self.blocks_databases,
                    &self.wot_databases.certs_db,
                    &Block::NetworkBlock(block),
                    &current_blockstamp,
                    wotb_index,
                    wot,
                    &self.forks_states,
                ) {
                current_blockstamp = block.blockstamp();
                // Update forks states
                self.forks_states = duniter_dal::block::get_forks(
                    &self.blocks_databases.forks_db,
                    current_blockstamp,
                ).expect("get_forks() : DALError");
                // Apply db requests
                bc_db_query
                    .apply(&self.blocks_databases, false)
                    .expect("Fatal error : Fail to apply DBWriteRequest !");
                wot_dbs_queries
                    .iter()
                    .map(|req| {
                        req.apply(&self.wot_databases, &self.currency_params)
                            .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
                    })
                    .collect::<()>();
                tx_dbs_queries
                    .iter()
                    .map(|req| {
                        req.apply(&self.currency_databases)
                            .expect("Fatal error : Fail to apply CurrencyDBsWriteRequest !");
                    })
                    .collect::<()>();
                save_blocks_dbs = true;
                if !wot_dbs_queries.is_empty() {
                    save_wots_dbs = true;
                }
                if !tx_dbs_queries.is_empty() {
                    save_currency_dbs = true;
                }
            }
        }
        // Save databases
        if save_blocks_dbs {
            self.blocks_databases.save_dbs();
        }
        if save_wots_dbs {
            self.wot_databases.save_dbs();
        }
        if save_currency_dbs {
            self.currency_databases.save_dbs(true, true);
        }
        current_blockstamp
    }
    /// Start blockchain module.
    pub fn start_blockchain(&mut self, blockchain_receiver: &mpsc::Receiver<DuniterMessage>) -> () {
        info!("BlockchainModule::start_blockchain()");

        // Get wot path
        let wot_path = duniter_conf::get_wot_path(self.conf_profile.clone(), &self.currency);

        // Get wotb index
        let mut wotb_index: HashMap<PubKey, NodeId> =
            DALIdentity::get_wotb_index(&self.wot_databases.identities_db)
                .expect("Fatal eror : get_wotb_index : Fail to read blockchain databases");

        // Open wot file
        let (mut wot, mut _wot_blockstamp) = open_wot_file::<RustyWebOfTrust, BinaryFileFormater>(
            &WOT_FILE_FORMATER,
            &wot_path,
            self.currency_params.sig_stock,
        );

        // Get current block
        let mut current_blockstamp = duniter_dal::block::get_current_blockstamp(
            &self.blocks_databases,
        ).expect("Fatal error : fail to read ForksV10DB !")
            .unwrap_or_default();

        // Init datas
        let mut last_get_stackables_blocks = UNIX_EPOCH;
        let mut last_request_blocks = UNIX_EPOCH;
        let mut pending_network_requests: HashMap<ModuleReqId, NetworkRequest> = HashMap::new();
        let mut consensus = Blockstamp::default();

        loop {
            // Request Consensus
            let req = NetworkRequest::GetConsensus(ModuleReqFullId(
                BlockchainModule::id(),
                ModuleReqId(pending_network_requests.len() as u32),
            ));
            let req_id = self.request_network(req);
            pending_network_requests.insert(req_id, req);
            // Request Blocks
            let now = SystemTime::now();
            if now
                .duration_since(last_request_blocks)
                .expect("duration_since error") > Duration::new(20, 0)
            {
                last_request_blocks = now;
                // Request begin blocks
                let to = match consensus.id.0 {
                    0 => (current_blockstamp.id.0 + *MAX_BLOCKS_REQUEST),
                    _ => consensus.id.0,
                };
                let new_pending_network_requests = self.request_blocks_to(
                    &pending_network_requests,
                    &current_blockstamp,
                    BlockId(to),
                );
                for (new_req_id, new_req) in new_pending_network_requests {
                    pending_network_requests.insert(new_req_id, new_req);
                }
                // Request end blocks
                if consensus.id.0 > (current_blockstamp.id.0 + 100) {
                    let mut req_id = ModuleReqId(0);
                    while pending_network_requests.contains_key(&req_id) {
                        req_id = ModuleReqId(req_id.0 + 1);
                    }
                    let from = consensus.id.0 - *CHUNK_SIZE - 1;
                    let (new_req_id, new_req) = self.request_chunk(&req_id, from);
                    pending_network_requests.insert(new_req_id, new_req);
                }
            }
            match blockchain_receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(ref message) => match *message {
                    DuniterMessage::Followers(ref new_followers) => {
                        info!("Blockchain module receive followers !");
                        for new_follower in new_followers {
                            self.followers.push(new_follower.clone());
                        }
                    }
                    DuniterMessage::DALRequest(ref dal_request) => match *dal_request {
                        DALRequest::BlockchainRequest(ref blockchain_req) => {
                            match *blockchain_req {
                                DALReqBlockchain::CurrentBlock(ref requester_full_id) => {
                                    debug!("BlockchainModule : receive DALReqBc::CurrentBlock()");

                                    if let Some(current_block) =
                                        DALBlock::get_block(
                                            &self.blocks_databases.blockchain_db,
                                            None,
                                            &current_blockstamp,
                                        ).expect(
                                            "Fatal error : get_block : fail to read LocalBlockchainV10DB !",
                                        ) {
                                        debug!("BlockchainModule : send_req_response(CurrentBlock({}))", current_blockstamp);
                                        self.send_req_response(&DALResponse::Blockchain(Box::new(
                                            DALResBlockchain::CurrentBlock(
                                                *requester_full_id,
                                                Box::new(current_block.block),
                                                current_blockstamp,
                                            ),
                                        )));
                                    } else {
                                        warn!("BlockchainModule : Req : fail to get current_block in bdd !");
                                    }
                                }
                                DALReqBlockchain::UIDs(ref pubkeys) => {
                                    self.send_req_response(&DALResponse::Blockchain(Box::new(
                                        DALResBlockchain::UIDs(
                                            pubkeys
                                                .iter()
                                                .map(|p| {
                                                    (
                                                        *p,
                                                        duniter_dal::identity::get_uid(&self.wot_databases.identities_db, *p)
                                                            .expect("Fatal error : get_uid : Fail to read WotV10DB !")
                                                    )
                                                })
                                                .collect(),
                                        ),
                                    )));
                                }
                                _ => {}
                            }
                        }
                        DALRequest::PendingsRequest(ref _pending_req) => {}
                    },
                    DuniterMessage::NetworkEvent(ref network_event) => match *network_event {
                        NetworkEvent::ReceiveDocuments(ref network_docs) => {
                            let new_current_blockstamp = self.receive_network_documents(
                                network_docs,
                                &current_blockstamp,
                                &mut wotb_index,
                                &mut wot,
                            );
                            current_blockstamp = new_current_blockstamp;
                        }
                        NetworkEvent::ReqResponse(ref network_response) => {
                            debug!("BlockchainModule : receive NetworkEvent::ReqResponse() !");
                            if let Some(request) =
                                pending_network_requests.remove(&network_response.get_req_id())
                            {
                                match request {
                                    NetworkRequest::GetConsensus(_) => {
                                        if let NetworkResponse::Consensus(_, response) =
                                            *network_response.deref()
                                        {
                                            if let Ok(blockstamp) = response {
                                                consensus = blockstamp;
                                            }
                                        }
                                    }
                                    NetworkRequest::GetBlocks(_, _, _, _) => {
                                        if let NetworkResponse::Chunk(_, _, ref blocks) =
                                            *network_response.deref()
                                        {
                                            let new_current_blockstamp = self.receive_blocks(
                                                blocks,
                                                &current_blockstamp,
                                                &mut wotb_index,
                                                &mut wot,
                                            );
                                            if current_blockstamp != new_current_blockstamp {
                                                current_blockstamp = new_current_blockstamp;
                                                // Update forks states
                                                self.forks_states =
                                                    duniter_dal::block::get_forks(
                                                        &self.blocks_databases.forks_db,
                                                        current_blockstamp,
                                                    ).expect("get_forks() : DALError");
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    },
                    DuniterMessage::ReceiveDocsFromClient(ref docs) => {
                        self.receive_documents(docs);
                    }
                    DuniterMessage::Stop() => break,
                    _ => {}
                },
                Err(e) => match e {
                    mpsc::RecvTimeoutError::Disconnected => {
                        panic!("Disconnected blockchain module !");
                    }
                    mpsc::RecvTimeoutError::Timeout => {}
                },
            }
            // Try to apply local stackable blocks
            let now = SystemTime::now();
            if now
                .duration_since(last_get_stackables_blocks)
                .expect("duration_since error") > Duration::new(20, 0)
            {
                last_get_stackables_blocks = now;
                loop {
                    let stackable_blocks = duniter_dal::block::DALBlock::get_stackables_blocks(
                        &self.blocks_databases.forks_db,
                        &self.blocks_databases.forks_blocks_db,
                        &current_blockstamp,
                    ).expect("Fatal error : Fail to read ForksV10DB !");
                    if stackable_blocks.is_empty() {
                        break;
                    } else {
                        let mut find_valid_block = false;
                        for stackable_block in stackable_blocks {
                            debug!("stackable_block({})", stackable_block.block.number);
                            if let Ok(ValidBlockApplyReqs(
                                bc_db_query,
                                wot_dbs_queries,
                                tx_dbs_queries,
                            )) = check_and_apply_block(
                                &self.blocks_databases,
                                &self.wot_databases.certs_db,
                                &Block::LocalBlock(&stackable_block.block),
                                &current_blockstamp,
                                &mut wotb_index,
                                &mut wot,
                                &self.forks_states,
                            ) {
                                // Apply db requests
                                bc_db_query
                                    .apply(&self.blocks_databases, false)
                                    .expect("Fatal error : Fail to apply DBWriteRequest !");
                                wot_dbs_queries
                                    .iter()
                                    .map(|req| {
                                        req.apply(&self.wot_databases, &self.currency_params)
                                            .expect(
                                                "Fatal error : Fail to apply WotsDBsWriteRequest !",
                                            );
                                    })
                                    .collect::<()>();
                                tx_dbs_queries
                                    .iter()
                                    .map(|req| {
                                        req.apply(&self.currency_databases).expect(
                                            "Fatal error : Fail to apply CurrencyDBsWriteRequest !",
                                        );
                                    })
                                    .collect::<()>();
                                // Save databases
                                self.blocks_databases.save_dbs();
                                if !wot_dbs_queries.is_empty() {
                                    self.wot_databases.save_dbs();
                                }
                                if !tx_dbs_queries.is_empty() {
                                    self.currency_databases.save_dbs(true, true);
                                }
                                debug!(
                                    "success to stackable_block({})",
                                    stackable_block.block.number
                                );

                                current_blockstamp = stackable_block.block.blockstamp();
                                find_valid_block = true;
                                break;
                            } else {
                                warn!(
                                    "DEBUG: fail to stackable_block({})",
                                    stackable_block.block.number
                                );
                                // Delete this fork
                                DALBlock::delete_fork(
                                    &self.blocks_databases.forks_db,
                                    &self.blocks_databases.forks_blocks_db,
                                    stackable_block.fork_id,
                                ).expect("delete_fork() : DALError");
                                // Update forks states
                                self.forks_states[stackable_block.fork_id.0] = ForkStatus::Free();
                            }
                        }
                        if !find_valid_block {
                            break;
                        }
                    }
                }
                // Print current_blockstamp
                info!(
                    "BlockchainModule : current_blockstamp() = {:?}",
                    current_blockstamp
                );
            }
            // Apply wot events
            /*BlockchainModule::apply_wot_events(
                &wot_events,
                &wot_path,
                &current_blockstamp,
                &mut wot,
                &mut wotb_index,
            );*/        }
    }
}

/// Complete Network Block
pub fn complete_network_block(
    network_block: &NetworkBlock,
) -> Result<BlockDocument, CompletedBlockError> {
    if let NetworkBlock::V10(ref network_block_v10) = *network_block {
        let mut block_doc = network_block_v10.uncompleted_block_doc.clone();
        trace!("complete_network_block #{}...", block_doc.number);
        block_doc.certifications =
            duniter_dal::parsers::certifications::parse_certifications_into_compact(
                &network_block_v10.certifications,
            );
        trace!("Success to complete certs.");
        block_doc.revoked = duniter_dal::parsers::revoked::parse_revocations_into_compact(
            &network_block_v10.revoked,
        );
        trace!("Success to complete certs & revocations.");
        let inner_hash = block_doc.inner_hash.expect(
            "BlockchainModule : complete_network_block() : fatal error : block.inner_hash = None",
        );
        if block_doc.number.0 > 0 {
            block_doc.compute_inner_hash();
        }
        let hash = block_doc.hash;
        block_doc.compute_hash();
        if block_doc.inner_hash.expect(
            "BlockchainModule : complete_network_block() : fatal error : block.inner_hash = None",
        ) == inner_hash
        {
            let nonce = block_doc.nonce;
            block_doc.change_nonce(nonce);
            if block_doc.hash == hash {
                trace!("Succes to complete_network_block #{}", block_doc.number.0);
                Ok(block_doc)
            } else {
                warn!("BlockchainModule : Refuse Bloc : invalid hash !");
                Err(CompletedBlockError::InvalidHash())
            }
        } else {
            warn!("BlockchainModule : Refuse Bloc : invalid inner hash !");
            debug!(
                "BlockInnerFormat={}",
                block_doc.generate_compact_inner_text()
            );
            Err(CompletedBlockError::InvalidInnerHash())
        }
    } else {
        Err(CompletedBlockError::InvalidVersion())
    }
}

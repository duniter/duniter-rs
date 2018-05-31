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

mod stack_up_block;
mod sync;

use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use self::stack_up_block::try_stack_up_completed_block;
use duniter_crypto::keys::*;
use duniter_dal::block::{DALBlock, WotEvent};
use duniter_dal::constants::MAX_FORKS;
use duniter_dal::dal_event::DALEvent;
use duniter_dal::dal_requests::{DALReqBlockchain, DALRequest, DALResBlockchain, DALResponse};
use duniter_dal::identity::DALIdentity;
use duniter_dal::parsers::memberships::MembershipParseError;
use duniter_dal::writers::requests::DBWriteRequest;
use duniter_dal::{DuniterDB, ForkState};
use duniter_documents::blockchain::v10::documents::{BlockDocument, V10Document};
use duniter_documents::blockchain::{BlockchainProtocol, Document, VerificationResult};
use duniter_documents::{BlockHash, BlockId, Blockstamp};
use duniter_message::DuniterMessage;
use duniter_module::*;
use duniter_network::{
    NetworkBlock, NetworkDocument, NetworkEvent, NetworkRequest, NetworkResponse, NodeFullId,
};
use duniter_wotb::data::rusty::RustyWebOfTrust;
use duniter_wotb::operations::file::{BinaryFileFormater, FileFormater};
use duniter_wotb::{NodeId, WebOfTrust};

/// The blocks are requested by packet groups. This constant sets the block packet size.
pub static CHUNK_SIZE: &'static u32 = &50;
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
    /// Database containing the blockchain
    pub db: DuniterDB,
    /// The block under construction
    pub pending_block: Option<Box<BlockDocument>>,
}

#[derive(Debug, Clone)]
/// Block
enum Block<'a> {
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
    /// MembershipParseError
    MembershipParseError(MembershipParseError),
    /// Invalid block inner hash
    InvalidInnerHash(),
    /// Invalid block signature
    InvalidSig(),
    /// Invalid block hash
    InvalidHash(),
    /// Invalid block version
    InvalidVersion(),
}

impl From<MembershipParseError> for CompletedBlockError {
    fn from(e: MembershipParseError) -> CompletedBlockError {
        CompletedBlockError::MembershipParseError(e)
    }
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
        sync: bool,
    ) -> BlockchainModule {
        // Get db path
        let db_path = duniter_conf::get_db_path(conf.profile().as_str(), &conf.currency(), sync);

        // Open duniter database
        let db = duniter_dal::open_db(&db_path, false).unwrap();

        // Instanciate BlockchainModule
        BlockchainModule {
            followers: Vec::new(),
            conf_profile: conf.profile(),
            currency: conf.currency(),
            db,
            pending_block: None,
        }
    }
    /// Synchronize blockchain from a duniter-ts database
    pub fn sync_ts(conf: &DuniterConf, ts_profile: &str, cautious: bool) {
        // Open local blockchain db
        let db_path = duniter_conf::get_db_path(&conf.profile(), &conf.currency(), false);
        let db = duniter_dal::open_db(&db_path, false).expect(&format!(
            "Fatal error : fail to open blockchain database as path : {} !",
            db_path.as_path().to_str().unwrap()
        ));
        // Get local current blockstamp
        debug!("Get local current blockstamp...");
        let current_block: Option<BlockDocument> = duniter_dal::new_get_current_block(&db);
        let current_blockstamp = match current_block.clone() {
            Some(block) => block.blockstamp(),
            None => Blockstamp::default(),
        };
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
        &self,
        network_documents: &[NetworkDocument],
        current_blockstamp: &Blockstamp,
        forks: &mut Vec<ForkState>,
        wotb_index: &HashMap<PubKey, NodeId>,
        wot: &W,
    ) -> (Blockstamp, Vec<WotEvent>) {
        let mut blockchain_documents = Vec::new();
        let mut current_blockstamp = *current_blockstamp;
        let mut wot_events = Vec::new();
        for network_document in network_documents {
            match *network_document {
                NetworkDocument::Block(ref network_block) => {
                    let (success, _new_forks, mut new_wot_events) = self.apply_block(
                        &Block::NetworkBlock(network_block),
                        &current_blockstamp,
                        forks,
                        wotb_index,
                        wot,
                    );
                    if success {
                        current_blockstamp = network_block.blockstamp();
                        wot_events.append(&mut new_wot_events);
                        // Update isolates forks
                        let stackables_forks =
                            DALBlock::get_stackables_forks(&self.db, &current_blockstamp);
                        for fork in stackables_forks {
                            debug!("unisolate fork {}", fork);
                            if forks.len() > fork {
                                forks[fork] = ForkState::Full();
                                DALBlock::unisolate_fork(&self.db, fork);
                            }
                        }
                    } /*else if !new_forks.is_empty() {
                        forks = new_forks;
                    }*/
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
        (current_blockstamp, wot_events)
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
        &self,
        blocks_in_box: &[Box<NetworkBlock>],
        current_blockstamp: &Blockstamp,
        forks: &[ForkState],
        wotb_index: &HashMap<PubKey, NodeId>,
        wot: &W,
    ) -> (Blockstamp, Vec<ForkState>, Vec<WotEvent>) {
        debug!("BlockchainModule : receive_blocks()");
        let blocks: Vec<&NetworkBlock> = blocks_in_box.into_iter().map(|b| b.deref()).collect();
        let mut current_blockstamp = *current_blockstamp;
        let mut all_wot_events = Vec::new();
        let mut forks = forks.to_owned();
        let mut wot_copy: W = wot.clone();
        let mut wotb_index_copy = wotb_index.clone();
        for block in blocks {
            let (success, _new_forks, mut wot_events) = self.apply_block::<W>(
                &Block::NetworkBlock(block),
                &current_blockstamp,
                &mut forks,
                &wotb_index_copy,
                &wot_copy,
            );
            all_wot_events.append(&mut wot_events);
            if success {
                current_blockstamp = block.blockstamp();
            } /*else if !new_forks.is_empty() {
                forks = new_forks;
            }*/
            if !wot_events.is_empty() {
                for wot_event in wot_events {
                    match wot_event {
                        WotEvent::AddNode(pubkey, wotb_id) => {
                            wot_copy.add_node();
                            wotb_index_copy.insert(pubkey, wotb_id);
                        }
                        WotEvent::RemNode(pubkey) => {
                            wot_copy.rem_node();
                            wotb_index_copy.remove(&pubkey);
                        }
                        WotEvent::AddLink(source, target) => {
                            wot_copy.add_link(source, target);
                        }
                        WotEvent::RemLink(source, target) => {
                            wot_copy.rem_link(source, target);
                        }
                        WotEvent::EnableNode(wotb_id) => {
                            wot_copy.set_enabled(wotb_id, true);
                        }
                        WotEvent::DisableNode(wotb_id) => {
                            wot_copy.set_enabled(wotb_id, false);
                        }
                    }
                }
            }
        }
        (current_blockstamp, forks, all_wot_events)
    }
    /*fn apply_local_block<W: WebOfTrust>(
        db: &sqlite::connexion,
        current_blockstamp: &Blockstamp,
        wotb_index: &HashMap<PubKey, NodeId>,
        wot: &W,
    ) {
        for f in 1..10 {
            let potential_next_block = get_block(db, );
        }
    }*/
    fn apply_block<W: WebOfTrust + Sync>(
        &self,
        block: &Block,
        current_blockstamp: &Blockstamp,
        forks: &mut Vec<ForkState>,
        wotb_index: &HashMap<PubKey, NodeId>,
        wot: &W,
    ) -> (bool, Vec<ForkState>, Vec<WotEvent>) {
        let mut already_have_block = false;
        let block_doc = match *block {
            Block::NetworkBlock(network_block) => match *network_block {
                NetworkBlock::V10(ref network_block_v10) => {
                    let (hashs, _) = DALBlock::get_blocks_hashs_all_forks(
                        &self.db,
                        &network_block_v10.uncompleted_block_doc.number,
                    );
                    for hash in hashs {
                        if hash == network_block_v10.uncompleted_block_doc.hash.unwrap() {
                            already_have_block = true;
                        }
                    }
                    &network_block_v10.uncompleted_block_doc
                }
                _ => return (false, Vec::with_capacity(0), Vec::with_capacity(0)),
            },
            Block::LocalBlock(block_doc) => {
                already_have_block = true;
                block_doc
            }
        };
        if (block_doc.number.0 == current_blockstamp.id.0 + 1
            && block_doc.previous_hash.to_string() == current_blockstamp.hash.0.to_string())
            || (block_doc.number.0 == 0 && *current_blockstamp == Blockstamp::default())
        {
            debug!(
                "stackable_block : block {} chainable !",
                block_doc.blockstamp()
            );
            let (success, db_requests, wot_events) = match *block {
                Block::NetworkBlock(network_block) => self.try_stack_up_block(
                    &network_block,
                    wotb_index,
                    wot,
                    SyncVerificationLevel::Cautious(),
                ),
                Block::LocalBlock(block_doc) => {
                    try_stack_up_completed_block(&block_doc, wotb_index, wot)
                }
            };
            debug!(
                "stackable_block_ : block {} chainable !",
                block_doc.blockstamp()
            );
            if success {
                // Apply db requests
                db_requests
                    .iter()
                    .map(|req| req.apply(&block_doc.currency, &self.db))
                    .collect::<()>();
                info!("StackUpValidBlock({})", block_doc.number.0);
                self.send_event(&DALEvent::StackUpValidBlock(Box::new(block_doc.clone())));
                return (true, Vec::with_capacity(0), wot_events);
            } else {
                warn!("RefusedBlock({})", block_doc.number.0);
                self.send_event(&DALEvent::RefusedPendingDoc(BlockchainProtocol::V10(
                    Box::new(V10Document::Block(Box::new(block_doc.clone()))),
                )));
            }
        } else if !already_have_block
            && (block_doc.number.0 >= current_blockstamp.id.0
                || (current_blockstamp.id.0 - block_doc.number.0) < 100)
        {
            debug!(
                "stackable_block : block {} not chainable, store this for future !",
                block_doc.blockstamp()
            );
            //let mut forks = forks.clone();
            let (fork, fork_state) = match DALBlock::get_block_fork(
                &self.db,
                &Blockstamp {
                    id: BlockId(block_doc.number.0 - 1),
                    hash: BlockHash(block_doc.previous_hash),
                },
            ) {
                Some(fork) => if forks.len() > fork {
                    (fork, forks[fork])
                } else {
                    panic!(format!("Error: fork n° {} is indicated as non-existent whereas it exists in database !", fork));
                },
                None => {
                    let mut free_fork = 0;
                    while forks.len() > free_fork && forks[free_fork] != ForkState::Free() {
                        free_fork += 1;
                    }
                    if free_fork >= *MAX_FORKS {
                        return (false, Vec::with_capacity(0), Vec::with_capacity(0));
                    }
                    info!("BlockchainModule : New Isolate fork : {}", free_fork);
                    if free_fork == forks.len() {
                        forks.push(ForkState::Isolate());
                        (forks.len() - 1, ForkState::Isolate())
                    } else {
                        forks[free_fork] = ForkState::Isolate();
                        (free_fork, ForkState::Isolate())
                    }
                }
            };
            let mut isolate = true;
            match fork_state {
                ForkState::Full() => isolate = false,
                ForkState::Isolate() => {}
                ForkState::Free() => {
                    warn!("fork n° {} is indicated as free when it is not !", fork);
                    forks[fork] = ForkState::Isolate();
                }
            }
            match *block {
                Block::NetworkBlock(network_block) => match *network_block {
                    NetworkBlock::V10(ref network_block_v10) => {
                        duniter_dal::writers::block::write_network_block(
                            &self.db,
                            &network_block_v10.uncompleted_block_doc,
                            fork,
                            isolate,
                            &network_block_v10.revoked,
                            &network_block_v10.certifications,
                        )
                    }
                    _ => return (false, Vec::with_capacity(0), Vec::with_capacity(0)),
                },
                Block::LocalBlock(block_doc) => {
                    duniter_dal::writers::block::write(&self.db, &block_doc, fork, isolate)
                }
            };
            return (false, forks.to_vec(), Vec::with_capacity(0));
        } else {
            debug!(
                "stackable_block : block {} not chainable and already stored !",
                block_doc.blockstamp()
            );
        }
        (false, Vec::with_capacity(0), Vec::with_capacity(0))
    }
    /// Try stack up block
    pub fn try_stack_up_block<W: WebOfTrust + Sync>(
        &self,
        network_block: &NetworkBlock,
        wotb_index: &HashMap<PubKey, NodeId>,
        wot: &W,
        verif_level: SyncVerificationLevel,
    ) -> (bool, Vec<DBWriteRequest>, Vec<WotEvent>) {
        let block_doc = match complete_network_block(
            &self.currency.to_string(),
            Some(&self.db),
            network_block,
            verif_level,
        ) {
            Ok(block_doc) => block_doc,
            Err(_) => return (false, Vec::with_capacity(0), Vec::with_capacity(0)),
        };
        try_stack_up_completed_block::<W>(&block_doc, wotb_index, wot)
    }
    /// Start blockchain module.
    pub fn start_blockchain(&mut self, blockchain_receiver: &mpsc::Receiver<DuniterMessage>) -> () {
        info!("BlockchainModule::start_blockchain()");

        // Get wot path
        let wot_path = duniter_conf::get_wot_path(self.conf_profile.clone(), &self.currency);

        // Get wotb index
        let mut wotb_index: HashMap<PubKey, NodeId> = DALIdentity::get_wotb_index(&self.db);

        // Open wot file
        let (mut wot, mut _wot_blockstamp) = duniter_dal::open_wot_file::<
            RustyWebOfTrust,
            BinaryFileFormater,
        >(&WOT_FILE_FORMATER, &wot_path);

        // Get forks
        let mut forks: Vec<ForkState> = duniter_dal::block::get_forks(&self.db);
        let mut last_get_stackables_blocks = UNIX_EPOCH;
        let mut last_request_blocks = UNIX_EPOCH;

        // Get current block
        let current_block: Option<BlockDocument> = duniter_dal::new_get_current_block(&self.db);
        let mut current_blockstamp = match current_block.clone() {
            Some(block) => block.blockstamp(),
            None => Blockstamp::default(),
        };

        // Init datas
        let mut pending_network_requests: HashMap<ModuleReqId, NetworkRequest> = HashMap::new();
        let mut consensus = Blockstamp::default();

        loop {
            let mut wot_events = Vec::new();
            // Request Consensus
            let req = NetworkRequest::GetConsensus(ModuleReqFullId(
                BlockchainModule::id(),
                ModuleReqId(pending_network_requests.len() as u32),
            ));
            let req_id = self.request_network(req);
            pending_network_requests.insert(req_id, req);
            // Request Blocks
            let now = SystemTime::now();
            if now.duration_since(last_request_blocks).unwrap() > Duration::new(20, 0) {
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

                                    if let Some(current_block) = DALBlock::get_block(
                                        &self.currency.to_string(),
                                        &self.db,
                                        &current_blockstamp,
                                    ) {
                                        debug!("BlockchainModule : send_req_response(CurrentBlock({}))", current_block.block.blockstamp());
                                        self.send_req_response(&DALResponse::Blockchain(Box::new(
                                            DALResBlockchain::CurrentBlock(
                                                *requester_full_id,
                                                Box::new(current_block.block),
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
                                                    if let Some(wotb_id) = wotb_index.get(p) {
                                                        (
                                                            *p,
                                                            duniter_dal::get_uid(
                                                                &self.db, *wotb_id,
                                                            ),
                                                        )
                                                    } else {
                                                        (*p, None)
                                                    }
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
                            let (new_current_blockstamp, mut new_wot_events) = self
                                .receive_network_documents(
                                    network_docs,
                                    &current_blockstamp,
                                    &mut forks,
                                    &wotb_index,
                                    &wot,
                                );
                            current_blockstamp = new_current_blockstamp;
                            wot_events.append(&mut new_wot_events);
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
                                            let (
                                                new_current_blockstamp,
                                                new_forks,
                                                mut new_wot_events,
                                            ) = self.receive_blocks(
                                                blocks,
                                                &current_blockstamp,
                                                &forks,
                                                &wotb_index,
                                                &wot,
                                            );
                                            current_blockstamp = new_current_blockstamp;
                                            wot_events.append(&mut new_wot_events);
                                            if !new_forks.is_empty() {
                                                forks = new_forks;
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
            // Write wot
            BlockchainModule::apply_wot_events(
                &wot_events,
                &wot_path,
                &current_blockstamp,
                &mut wot,
                &mut wotb_index,
            );
            // Try to apply local stackable blocks
            let mut wot_events = Vec::new();
            let now = SystemTime::now();
            if now.duration_since(last_get_stackables_blocks).unwrap() > Duration::new(20, 0) {
                last_get_stackables_blocks = now;
                loop {
                    let stackable_blocks = duniter_dal::block::DALBlock::get_stackables_blocks(
                        &self.currency.to_string(),
                        &self.db,
                        &current_blockstamp,
                    );
                    if stackable_blocks.is_empty() {
                        break;
                    } else {
                        let mut find_valid_block = false;
                        for stackable_block in stackable_blocks {
                            debug!("stackable_block({})", stackable_block.block.number);
                            let (success, _new_forks, mut new_wot_events) = self.apply_block(
                                &Block::LocalBlock(&stackable_block.block),
                                &current_blockstamp,
                                &mut forks,
                                &wotb_index,
                                &wot,
                            );
                            if success {
                                debug!(
                                    "success to stackable_block({})",
                                    stackable_block.block.number
                                );
                                current_blockstamp = stackable_block.block.blockstamp();
                                wot_events.append(&mut new_wot_events);
                                find_valid_block = true;
                                /*if !new_forks.is_empty() {
                                    forks = new_forks;
                                }*/
                                break;
                            } else {
                                warn!(
                                    "DEBUG: fail to stackable_block({})",
                                    stackable_block.block.number
                                );
                                // Delete this fork
                                DALBlock::delete_fork(&self.db, stackable_block.fork);
                                forks[stackable_block.fork] = ForkState::Free();
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
            // Write wot
            BlockchainModule::apply_wot_events(
                &wot_events,
                &wot_path,
                &current_blockstamp,
                &mut wot,
                &mut wotb_index,
            );
        }
    }
    fn apply_wot_events<W: WebOfTrust + Sync>(
        wot_events: &[WotEvent],
        wot_path: &PathBuf,
        current_blockstamp: &Blockstamp,
        wot: &mut W,
        wotb_index: &mut HashMap<PubKey, NodeId>,
    ) {
        if !wot_events.is_empty() {
            for wot_event in wot_events {
                match *wot_event {
                    WotEvent::AddNode(pubkey, wotb_id) => {
                        wot.add_node();
                        wotb_index.insert(pubkey, wotb_id);
                    }
                    WotEvent::RemNode(pubkey) => {
                        wot.rem_node();
                        wotb_index.remove(&pubkey);
                    }
                    WotEvent::AddLink(source, target) => {
                        wot.add_link(source, target);
                    }
                    WotEvent::RemLink(source, target) => {
                        wot.rem_link(source, target);
                    }
                    WotEvent::EnableNode(wotb_id) => {
                        wot.set_enabled(wotb_id, true);
                    }
                    WotEvent::DisableNode(wotb_id) => {
                        wot.set_enabled(wotb_id, false);
                    }
                }
            }
            // Save wot
            WOT_FILE_FORMATER
                .to_file(
                    wot,
                    current_blockstamp.to_string().as_bytes(),
                    wot_path.as_path().to_str().unwrap(),
                )
                .expect("Fatal Error: Fail to write wotb in file !");
        }
    }
}

/// Complete Network Block
pub fn complete_network_block(
    currency: &str,
    db: Option<&DuniterDB>,
    network_block: &NetworkBlock,
    verif_level: SyncVerificationLevel,
) -> Result<BlockDocument, CompletedBlockError> {
    if let NetworkBlock::V10(ref network_block_v10) = *network_block {
        let mut block_doc = network_block_v10.uncompleted_block_doc.clone();
        trace!("complete_network_block #{}...", block_doc.number);
        if verif_level == SyncVerificationLevel::Cautious() {
            // Indexing block_identities
            let mut block_identities = HashMap::new();
            block_doc
                .identities
                .iter()
                .map(|idty| {
                    if idty.issuers().is_empty() {
                        panic!("idty without issuer !")
                    }
                    block_identities.insert(idty.issuers()[0], idty.clone());
                })
                .collect::<()>();
            block_doc.certifications =
                    duniter_dal::parsers::certifications::parse_certifications_from_json_value(
                        currency,
                        db.expect("complete_network_block() : Cautious mode need access to blockchain database !"),
                        &block_identities,
                        &network_block_v10.certifications,
                    );
            trace!("Success to complete certs.");
            block_doc.revoked = duniter_dal::parsers::revoked::parse_revocations_from_json_value(
                currency,
                db.expect(
                    "complete_network_block() : Cautious mode need access to blockchain database !",
                ),
                &block_identities,
                &network_block_v10.revoked,
            );
        } else {
            block_doc.certifications =
                duniter_dal::parsers::certifications::parse_certifications_into_compact(
                    &network_block_v10.certifications,
                );
            trace!("Success to complete certs.");
            block_doc.revoked = duniter_dal::parsers::revoked::parse_revocations_into_compact(
                &network_block_v10.revoked,
            );
        }
        trace!("Success to complete certs & revocations.");
        // In cautions mode, verify all signatures !
        if verif_level == SyncVerificationLevel::Cautious() {
            for idty in block_doc.clone().identities {
                if idty.verify_signatures() != VerificationResult::Valid() {
                    error!(
                        "Fail to sync block #{} : Idty with invalid singature !",
                        block_doc.number
                    );
                    panic!("Idty with invalid singature !");
                }
            }
        }
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
            if verif_level == SyncVerificationLevel::FastSync()
                || block_doc.verify_signatures() == VerificationResult::Valid()
                || block_doc.number.0 <= 1
            {
                if block_doc.hash == hash {
                    trace!("Succes to complete_network_block #{}", block_doc.number.0);
                    Ok(block_doc)
                } else {
                    warn!("BlockchainModule : Refuse Bloc : invalid hash !");
                    Err(CompletedBlockError::InvalidHash())
                }
            } else {
                warn!("BlockchainModule : Refuse Bloc : invalid signature !");
                Err(CompletedBlockError::InvalidSig())
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

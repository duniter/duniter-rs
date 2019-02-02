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
#![allow(clippy::large_enum_variant)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

//#[macro_use]
//extern crate failure;
#[macro_use]
extern crate log;

mod apply_valid_block;
mod check_and_apply_block;
mod constants;
mod dbex;
mod revert_block;
mod sync;
mod verify_block;

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::str;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::apply_valid_block::*;
use crate::check_and_apply_block::*;
use crate::constants::*;
pub use crate::dbex::{DBExQuery, DBExTxQuery, DBExWotQuery};
use dubp_documents::documents::block::BlockDocument;
use dubp_documents::documents::DUBPDocument;
use dubp_documents::*;
use duniter_module::*;
use duniter_network::{
    cli::sync::SyncOpt,
    documents::BlockchainDocument,
    events::NetworkEvent,
    requests::{NetworkResponse, OldNetworkRequest},
};
use dup_crypto::keys::*;
use durs_blockchain_dal::block::DALBlock;
use durs_blockchain_dal::currency_params::CurrencyParameters;
use durs_blockchain_dal::identity::DALIdentity;
use durs_blockchain_dal::writers::requests::BlocksDBsWriteQuery;
use durs_blockchain_dal::*;
use durs_common_tools::fatal_error;
use durs_message::events::*;
use durs_message::requests::*;
use durs_message::responses::*;
use durs_message::*;
use durs_network_documents::NodeFullId;
use durs_wot::data::rusty::RustyWebOfTrust;
use durs_wot::operations::distance::RustyDistanceCalculator;
use durs_wot::{NodeId, WebOfTrust};

/// The blocks are requested by packet groups. This constant sets the block packet size.
pub static CHUNK_SIZE: &'static u32 = &50;
/// Necessary to instantiate the wot object before knowing the currency parameters
pub static INFINITE_SIG_STOCK: &'static usize = &4_000_000_000;
/// The blocks are requested by packet groups. This constant sets the number of packets per group.
pub static MAX_BLOCKS_REQUEST: &'static u32 = &500;
/// The distance calculator
pub static DISTANCE_CALCULATOR: &'static RustyDistanceCalculator = &RustyDistanceCalculator {};

/// Blockchain Module
#[derive(Debug)]
pub struct BlockchainModule {
    /// Router sender
    pub router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
    /// Name of the user datas profile
    pub profile: String,
    /// Currency
    pub currency: CurrencyName,
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
pub enum Block {
    /// Block coming from Network
    NetworkBlock(BlockDocument),
    /// Block coming from local database
    LocalBlock(BlockDocument),
}

impl Block {
    /// Into block document
    pub fn into_doc(self) -> BlockDocument {
        match self {
            Block::NetworkBlock(block) => block,
            Block::LocalBlock(block) => block,
        }
    }
    /// Get block document ref
    pub fn get_doc_ref(&self) -> &BlockDocument {
        match *self {
            Block::NetworkBlock(ref block) => block,
            Block::LocalBlock(ref block) => block,
        }
    }
    /// Return blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        match *self {
            Block::NetworkBlock(ref block) => block.blockstamp(),
            Block::LocalBlock(ref block) => block.blockstamp(),
        }
    }
    /// Is from network ?
    pub fn is_from_network(&self) -> bool {
        match *self {
            Block::NetworkBlock(_) => true,
            _ => false,
        }
    }
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
/// Error returned by function verify_block_hashs()
pub enum VerifyBlockHashsError {
    /// Invalid block inner hash
    InvalidInnerHash(),
    /// Invalid block hash
    InvalidHash(BlockId, Option<BlockHash>),
    /// Invalid block version
    InvalidVersion(),
}

impl BlockchainModule {
    /// Return module identifier
    pub fn name() -> ModuleStaticName {
        ModuleStaticName("blockchain")
    }
    /// Loading blockchain configuration
    pub fn load_blockchain_conf<DC: DuniterConf>(
        router_sender: mpsc::Sender<RouterThreadMessage<DursMsg>>,
        profile: &str,
        conf: &DC,
        _keys: RequiredKeysContent,
    ) -> BlockchainModule {
        // Get db path
        let db_path = duniter_conf::get_blockchain_db_path(profile, &conf.currency());

        // Open databases
        let blocks_databases = BlocksV10DBs::open(Some(&db_path));
        let wot_databases = WotsV10DBs::open(Some(&db_path));
        let currency_databases = CurrencyV10DBs::open(Some(&db_path));

        // Get current blockstamp
        let current_blockstamp =
            durs_blockchain_dal::block::get_current_blockstamp(&blocks_databases)
                .expect("Fatal error : fail to read Blockchain DB !");

        // Get currency parameters
        let currency_params = durs_blockchain_dal::currency_params::get_currency_params(
            &blocks_databases.blockchain_db,
        )
        .expect("Fatal error : fail to read Blockchain DB !")
        .unwrap_or_default();

        // Get forks states
        let forks_states = if let Some(current_blockstamp) = current_blockstamp {
            durs_blockchain_dal::block::get_forks(&blocks_databases.forks_db, current_blockstamp)
                .expect("Fatal error : fail to read Forks DB !")
        } else {
            vec![]
        };

        // Instanciate BlockchainModule
        BlockchainModule {
            router_sender,
            profile: profile.to_string(),
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
    pub fn dbex<DC: DuniterConf>(profile: &str, conf: &DC, csv: bool, req: &DBExQuery) {
        dbex::dbex(profile, conf, csv, req);
    }
    /// Synchronize blockchain from local duniter json files
    pub fn sync_ts<DC: DuniterConf>(profile: &str, conf: &DC, sync_opts: &SyncOpt) {
        // get json_chunks_path
        let json_chunks_path = if let Some(ref path) = sync_opts.source {
            PathBuf::from(path)
        } else {
            let mut json_chunks_path = match dirs::config_dir() {
                Some(path) => path,
                None => panic!("Impossible to get user config directory !"),
            };
            json_chunks_path.push("duniter/");
            json_chunks_path.push("duniter_default");

            let currency = if let Some(currency) = &sync_opts.currency {
                currency
            } else {
                DEFAULT_CURRENCY
            };

            json_chunks_path.push(currency);
            json_chunks_path
        };
        if !json_chunks_path.as_path().exists() {
            panic!("Fatal error : duniter json chunks folder don't exist !");
        }
        sync::local_sync(
            profile,
            conf,
            json_chunks_path,
            sync_opts.end,
            sync_opts.cautious_mode,
            !sync_opts.unsafe_mode,
        );
    }
    /// Request chunk from network (chunk = group of blocks)
    fn request_chunk(&self, req_id: ModuleReqId, from: u32) -> (ModuleReqId, OldNetworkRequest) {
        let req = OldNetworkRequest::GetBlocks(
            ModuleReqFullId(BlockchainModule::name(), req_id),
            NodeFullId::default(),
            *CHUNK_SIZE,
            from,
        );
        (self.request_network(req_id, &req), req)
    }
    /// Requests blocks from current to `to`
    fn request_blocks_to(
        &self,
        pending_network_requests: &HashMap<ModuleReqId, OldNetworkRequest>,
        current_blockstamp: &Blockstamp,
        to: BlockId,
    ) -> HashMap<ModuleReqId, OldNetworkRequest> {
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
                let (req_id, req) = self.request_chunk(req_id, from);
                requests_ids.insert(req_id, req);
                from += *CHUNK_SIZE;
            }
        }
        requests_ids
    }
    /// Send network request
    fn request_network(&self, req_id: ModuleReqId, request: &OldNetworkRequest) -> ModuleReqId {
        self.router_sender
            .send(RouterThreadMessage::ModuleMessage(DursMsg::Request {
                req_from: BlockchainModule::name(),
                req_to: ModuleRole::InterNodesNetwork,
                req_id,
                req_content: DursReqContent::OldNetworkRequest(*request),
            }))
            .unwrap_or_else(|_| panic!("Fail to send OldNetworkRequest to router"));
        request.get_req_id()
    }
    /// Send blockchain event
    fn send_event(&self, event: &BlockchainEvent) {
        let module_event = match event {
            BlockchainEvent::StackUpValidBlock(_, _) => ModuleEvent::NewValidBlock,
            BlockchainEvent::RevertBlocks(_) => ModuleEvent::RevertBlocks,
            _ => return,
        };
        self.router_sender
            .send(RouterThreadMessage::ModuleMessage(DursMsg::Event {
                event_type: module_event,
                event_content: DursEvent::BlockchainEvent(Box::new(event.clone())),
            }))
            .unwrap_or_else(|_| panic!("Fail to send BlockchainEvent to router"));
    }
    fn send_req_response(
        &self,
        requester: ModuleStaticName,
        req_id: ModuleReqId,
        response: &BlockchainResponse,
    ) {
        self.router_sender
            .send(RouterThreadMessage::ModuleMessage(DursMsg::Response {
                res_from: BlockchainModule::name(),
                res_to: requester,
                req_id,
                res_content: DursResContent::BlockchainResponse(response.clone()),
            }))
            .unwrap_or_else(|_| panic!("Fail to send ReqRes to router"));
    }
    fn receive_network_documents<W: WebOfTrust>(
        &mut self,
        network_documents: &[BlockchainDocument],
        current_blockstamp: &Blockstamp,
        wot_index: &mut HashMap<PubKey, NodeId>,
        wot_db: &BinDB<W>,
    ) -> Blockstamp {
        let mut current_blockstamp = *current_blockstamp;

        for network_document in network_documents {
            if let BlockchainDocument::Block(ref block_doc) = network_document {
                let block_doc = block_doc.deref();
                match check_and_apply_block::<W>(
                    &self.blocks_databases,
                    &self.wot_databases.certs_db,
                    Block::NetworkBlock(block_doc.clone()),
                    &current_blockstamp,
                    wot_index,
                    wot_db,
                    &self.forks_states,
                ) {
                    Ok(ValidBlockApplyReqs(block_req, wot_dbs_reqs, currency_dbs_reqs)) => {
                        let block_doc = block_doc.clone();
                        let mut save_wots_dbs = false;
                        let mut save_currency_dbs = false;

                        // Apply wot dbs requests
                        for req in &wot_dbs_reqs {
                            req.apply(&self.wot_databases, &self.currency_params)
                                .expect(
                                    "Fatal error : fail to apply WotsDBsWriteQuery : DALError !",
                                );
                        }
                        // Apply currency dbs requests
                        for req in currency_dbs_reqs {
                            req.apply(&self.currency_databases).expect(
                                "Fatal error : fail to apply CurrencyDBsWriteQuery : DALError !",
                            );
                        }
                        // Write block
                        block_req
                            .apply(&self.blocks_databases, false)
                            .expect("Fatal error : fail to write block in BlocksDBs : DALError !");
                        if let BlocksDBsWriteQuery::WriteBlock(_, _, _, block_hash) = block_req {
                            info!("StackUpValidBlock({})", block_doc.number.0);
                            self.send_event(&BlockchainEvent::StackUpValidBlock(
                                Box::new(block_doc.clone()),
                                Blockstamp {
                                    id: block_doc.number,
                                    hash: block_hash,
                                },
                            ));
                        }
                        current_blockstamp = block_doc.blockstamp();
                        // Update forks states
                        self.forks_states = durs_blockchain_dal::block::get_forks(
                            &self.blocks_databases.forks_db,
                            current_blockstamp,
                        )
                        .expect("get_forks() : DALError");

                        if !wot_dbs_reqs.is_empty() {
                            save_wots_dbs = true;
                        }
                        if !block_doc.transactions.is_empty()
                            || (block_doc.dividend.is_some()
                                && block_doc.dividend.expect("safe unwrap") > 0)
                        {
                            save_currency_dbs = true;
                        }

                        // Save databases
                        self.blocks_databases.save_dbs();
                        if save_wots_dbs {
                            self.wot_databases.save_dbs();
                        }
                        if save_currency_dbs {
                            self.currency_databases.save_dbs(true, true);
                        }
                    }
                    Err(_) => {
                        warn!("RefusedBlock({})", block_doc.number.0);
                        self.send_event(&BlockchainEvent::RefusedPendingDoc(DUBPDocument::Block(
                            Box::new(block_doc.clone()),
                        )));
                    }
                }
            }
        }

        current_blockstamp
    }

    fn receive_blocks<W: WebOfTrust>(
        &mut self,
        blocks: Vec<Block>,
        current_blockstamp: &Blockstamp,
        wot_index: &mut HashMap<PubKey, NodeId>,
        wot: &BinDB<W>,
    ) -> Blockstamp {
        debug!("BlockchainModule : receive_blocks()");
        let mut current_blockstamp = *current_blockstamp;
        let mut save_blocks_dbs = false;
        let mut save_wots_dbs = false;
        let mut save_currency_dbs = false;
        for block in blocks.into_iter() {
            let blockstamp = block.blockstamp();
            if let Ok(ValidBlockApplyReqs(bc_db_query, wot_dbs_queries, tx_dbs_queries)) =
                check_and_apply_block::<W>(
                    &self.blocks_databases,
                    &self.wot_databases.certs_db,
                    block,
                    &current_blockstamp,
                    wot_index,
                    wot,
                    &self.forks_states,
                )
            {
                current_blockstamp = blockstamp;
                // Update forks states
                self.forks_states = durs_blockchain_dal::block::get_forks(
                    &self.blocks_databases.forks_db,
                    current_blockstamp,
                )
                .expect("get_forks() : DALError");
                // Apply db requests
                bc_db_query
                    .apply(&self.blocks_databases, false)
                    .expect("Fatal error : Fail to apply DBWriteRequest !");
                for query in &wot_dbs_queries {
                    query
                        .apply(&self.wot_databases, &self.currency_params)
                        .expect("Fatal error : Fail to apply WotsDBsWriteRequest !");
                }
                for query in &tx_dbs_queries {
                    query
                        .apply(&self.currency_databases)
                        .expect("Fatal error : Fail to apply CurrencyDBsWriteRequest !");
                }
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
    pub fn start_blockchain(&mut self, blockchain_receiver: &mpsc::Receiver<DursMsg>) {
        info!("BlockchainModule::start_blockchain()");

        // Get dbs path
        let dbs_path = duniter_conf::get_blockchain_db_path(self.profile.as_str(), &self.currency);

        // Get wot index
        let mut wot_index: HashMap<PubKey, NodeId> =
            DALIdentity::get_wot_index(&self.wot_databases.identities_db)
                .expect("Fatal eror : get_wot_index : Fail to read blockchain databases");

        // Open wot file
        let wot_db = open_wot_db::<RustyWebOfTrust>(Some(&dbs_path)).expect("Fail to open WotDB !");

        // Get current block
        let mut current_blockstamp =
            durs_blockchain_dal::block::get_current_blockstamp(&self.blocks_databases)
                .expect("Fatal error : fail to read ForksV10DB !")
                .unwrap_or_default();

        // Init datas
        let mut last_get_stackables_blocks = UNIX_EPOCH;
        let mut last_request_blocks = UNIX_EPOCH;
        let mut pending_network_requests: HashMap<ModuleReqId, OldNetworkRequest> = HashMap::new();
        let mut consensus = Blockstamp::default();

        loop {
            // Request Consensus
            let req = OldNetworkRequest::GetConsensus(ModuleReqFullId(
                BlockchainModule::name(),
                ModuleReqId(pending_network_requests.len() as u32),
            ));
            let req_id =
                self.request_network(ModuleReqId(pending_network_requests.len() as u32), &req);
            pending_network_requests.insert(req_id, req);
            // Request Blocks
            let now = SystemTime::now();
            if now
                .duration_since(last_request_blocks)
                .expect("duration_since error")
                > Duration::new(20, 0)
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
                    let (new_req_id, new_req) = self.request_chunk(req_id, from);
                    pending_network_requests.insert(new_req_id, new_req);
                }
            }
            match blockchain_receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(ref durs_message) => {
                    match *durs_message {
                        DursMsg::Request {
                            req_from,
                            ref req_content,
                            ..
                        } => {
                            if let DursReqContent::BlockchainRequest(ref blockchain_req) =
                                req_content
                            {
                                match *blockchain_req {
                                    BlockchainRequest::CurrentBlock() => {
                                        debug!(
                                            "BlockchainModule : receive DALReqBc::CurrentBlock()"
                                        );

                                        if let Some(current_block) =
                                            DALBlock::get_block(
                                                &self.blocks_databases.blockchain_db,
                                                None,
                                                &current_blockstamp,
                                            ).expect(
                                                "Fatal error : get_block : fail to read LocalBlockchainV10DB !",
                                            )
                                        {
                                            debug!("BlockchainModule : send_req_response(CurrentBlock({}))", current_blockstamp);
                                            self.send_req_response(req_from, req_id, &BlockchainResponse::CurrentBlock(
                                                    req_id,
                                                    Box::new(current_block.block),
                                                    current_blockstamp,
                                                ),
                                            );
                                        } else {
                                                    warn!("BlockchainModule : Req : fail to get current_block in bdd !");
                                        }
                                    }
                                    BlockchainRequest::UIDs(ref pubkeys) => {
                                        self.send_req_response(req_from, req_id, &BlockchainResponse::UIDs(
                                                req_id,
                                                pubkeys
                                                    .iter()
                                                    .map(|p| (
                                                        *p,
                                                        durs_blockchain_dal::identity::get_uid(&self.wot_databases.identities_db, *p)
                                                            .expect("Fatal error : get_uid : Fail to read WotV10DB !")
                                                    ))
                                                    .collect(),
                                            ),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        DursMsg::Event {
                            ref event_content, ..
                        } => match *event_content {
                            DursEvent::NetworkEvent(ref network_event_box) => {
                                match *network_event_box.deref() {
                                    NetworkEvent::ReceiveDocuments(ref network_docs) => {
                                        let new_current_blockstamp = self
                                            .receive_network_documents(
                                                network_docs,
                                                &current_blockstamp,
                                                &mut wot_index,
                                                &wot_db,
                                            );
                                        current_blockstamp = new_current_blockstamp;
                                    }
                                    NetworkEvent::ReceiveHeads(_) => {}
                                    _ => {}
                                }
                            }
                            DursEvent::MemPoolEvent(ref mempool_event) => {
                                if let MemPoolEvent::FindNextBlock(next_block_box) = mempool_event {
                                    let new_current_blockstamp = self.receive_blocks(
                                        vec![Block::LocalBlock(next_block_box.deref().clone())],
                                        &current_blockstamp,
                                        &mut wot_index,
                                        &wot_db,
                                    );
                                    current_blockstamp = new_current_blockstamp;
                                }
                            }
                            _ => {} // Others modules events
                        },
                        DursMsg::Response {
                            ref req_id,
                            ref res_content,
                            ..
                        } => {
                            if let DursResContent::NetworkResponse(ref network_response) =
                                *res_content
                            {
                                debug!("BlockchainModule : receive NetworkResponse() !");
                                if let Some(request) = pending_network_requests.remove(req_id) {
                                    match request {
                                        OldNetworkRequest::GetConsensus(_) => {
                                            if let NetworkResponse::Consensus(_, response) =
                                                *network_response.deref()
                                            {
                                                if let Ok(blockstamp) = response {
                                                    consensus = blockstamp;
                                                    if current_blockstamp.id.0 > consensus.id.0 + 2
                                                    {
                                                        // Find free fork id
                                                        let free_fork_id = ForkId(49);
                                                        // Get last dal_block
                                                        let last_dal_block_id =
                                                            BlockId(current_blockstamp.id.0 - 1);
                                                        let last_dal_block = self
                                                        .blocks_databases
                                                        .blockchain_db
                                                        .read(|db| db.get(&last_dal_block_id).cloned())
                                                        .expect("Fail to read blockchain DB.")
                                                        .expect(
                                                            "Fatal error : not foutn last dal block !",
                                                        );
                                                        revert_block::revert_block(
                                                            &last_dal_block,
                                                            &mut wot_index,
                                                            &wot_db,
                                                            Some(free_fork_id),
                                                            &self
                                                                .currency_databases
                                                                .tx_db
                                                                .read(|db| db.clone())
                                                                .expect("Fail to read TxDB."),
                                                        )
                                                        .expect("Fail to revert block");
                                                    }
                                                }
                                            }
                                        }
                                        OldNetworkRequest::GetBlocks(_, _, _, _) => {
                                            if let NetworkResponse::Chunk(_, _, ref blocks) =
                                                *network_response.deref()
                                            {
                                                let blocks: Vec<Block> = blocks
                                                    .iter()
                                                    .map(|b| Block::NetworkBlock(b.deref().clone()))
                                                    .collect();

                                                let new_current_blockstamp = self.receive_blocks(
                                                    blocks,
                                                    &current_blockstamp,
                                                    &mut wot_index,
                                                    &wot_db,
                                                );
                                                if current_blockstamp != new_current_blockstamp {
                                                    current_blockstamp = new_current_blockstamp;
                                                    // Update forks states
                                                    self.forks_states =
                                                        durs_blockchain_dal::block::get_forks(
                                                            &self.blocks_databases.forks_db,
                                                            current_blockstamp,
                                                        )
                                                        .expect("get_forks() : DALError");
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        DursMsg::Stop => break,
                        _ => {} // Others DursMsg variants
                    }
                }
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
                .expect("duration_since error")
                > Duration::new(20, 0)
            {
                last_get_stackables_blocks = now;
                loop {
                    let stackable_blocks =
                        durs_blockchain_dal::block::DALBlock::get_stackables_blocks(
                            &self.blocks_databases.forks_db,
                            &self.blocks_databases.forks_blocks_db,
                            &current_blockstamp,
                        )
                        .expect("Fatal error : Fail to read ForksV10DB !");
                    if stackable_blocks.is_empty() {
                        break;
                    } else {
                        let mut find_valid_block = false;
                        for stackable_block in stackable_blocks {
                            debug!("stackable_block({})", stackable_block.block.number);

                            let stackable_block_number = stackable_block.block.number;
                            let stackable_block_blockstamp = stackable_block.block.blockstamp();

                            if let Ok(ValidBlockApplyReqs(
                                bc_db_query,
                                wot_dbs_queries,
                                tx_dbs_queries,
                            )) = check_and_apply_block(
                                &self.blocks_databases,
                                &self.wot_databases.certs_db,
                                Block::LocalBlock(stackable_block.block),
                                &current_blockstamp,
                                &mut wot_index,
                                &wot_db,
                                &self.forks_states,
                            ) {
                                // Apply db requests
                                bc_db_query
                                    .apply(&self.blocks_databases, false)
                                    .expect("Fatal error : Fail to apply DBWriteRequest !");
                                for query in &wot_dbs_queries {
                                    query
                                        .apply(&self.wot_databases, &self.currency_params)
                                        .expect(
                                            "Fatal error : Fail to apply WotsDBsWriteRequest !",
                                        );
                                }
                                for query in &tx_dbs_queries {
                                    query.apply(&self.currency_databases).expect(
                                        "Fatal error : Fail to apply CurrencyDBsWriteRequest !",
                                    );
                                }
                                // Save databases
                                self.blocks_databases.save_dbs();
                                if !wot_dbs_queries.is_empty() {
                                    self.wot_databases.save_dbs();
                                }
                                if !tx_dbs_queries.is_empty() {
                                    self.currency_databases.save_dbs(true, true);
                                }
                                debug!("success to stackable_block({})", stackable_block_number);

                                current_blockstamp = stackable_block_blockstamp;
                                find_valid_block = true;
                                break;
                            } else {
                                warn!("fail to stackable_block({})", stackable_block_number);
                                // Delete this fork
                                DALBlock::delete_fork(
                                    &self.blocks_databases.forks_db,
                                    &self.blocks_databases.forks_blocks_db,
                                    stackable_block.fork_id,
                                )
                                .expect("delete_fork() : DALError");
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
        }
    }
}

/// Verify block hashs
pub fn verify_block_hashs(block_doc: &BlockDocument) -> Result<(), VerifyBlockHashsError> {
    trace!("complete_block #{}...", block_doc.number);

    if block_doc.inner_hash.is_none() {
        fatal_error(
            "BlockchainModule : verify_block_hashs() : fatal error : block.inner_hash = None",
        );
    }

    if block_doc.verify_inner_hash() {
        if block_doc.verify_hash() {
            trace!("Succes to verify_block_hashs #{}", block_doc.number.0);
            Ok(())
        } else {
            warn!("BlockchainModule : Refuse Bloc : invalid hash !");
            Err(VerifyBlockHashsError::InvalidHash(
                block_doc.number,
                block_doc.hash,
            ))
        }
    } else {
        warn!("BlockchainModule : Refuse Bloc : invalid inner hash !");
        warn!("BlockDocument=\"{:?}\"", block_doc);
        warn!(
            "BlockInnerFormat=\"{}\"",
            block_doc.generate_compact_inner_text()
        );
        Err(VerifyBlockHashsError::InvalidInnerHash())
    }
}

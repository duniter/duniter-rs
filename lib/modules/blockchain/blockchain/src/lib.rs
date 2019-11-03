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

//! Module managing the Duniter blockchain.

#![allow(clippy::large_enum_variant)]
#![deny(
    missing_docs,
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

mod constants;
pub mod dbex;
mod dubp;
mod dunp;
mod events;
mod fork;
mod requests;
mod responses;
mod sync;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::constants::*;
use crate::dbex::DbExQuery;
use crate::dubp::apply::ValidBlockApplyReqs;
use crate::dubp::*;
use crate::fork::*;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::traits::Document;
use dubp_common_doc::Blockstamp;
use dubp_currency_params::{CurrencyName, CurrencyParameters};
use dup_crypto::keys::*;
use durs_bc_db_reader::blocks::fork_tree::ForkTree;
use durs_bc_db_writer::*;
use durs_common_tools::fatal_error;
use durs_message::events::*;
use durs_message::requests::*;
use durs_message::responses::*;
use durs_message::*;
use durs_module::*;
use durs_network::{
    cli::sync::SyncOpt,
    events::NetworkEvent,
    requests::{NetworkResponse, OldNetworkRequest},
};
// use durs_wot::data::rusty::RustyWebOfTrust;
use durs_wot::operations::distance::RustyDistanceCalculator;
use durs_wot::WotId;
use failure::Error;

/// The blocks are requested by packet groups. This constant sets the block packet size.
pub static CHUNK_SIZE: &u32 = &50;
/// Necessary to instantiate the wot object before knowing the currency parameters
pub static INFINITE_SIG_STOCK: &usize = &4_000_000_000;
/// The blocks are requested by packet groups. This constant sets the number of packets per group.
pub static MAX_BLOCKS_REQUEST: &u32 = &500;
/// The distance calculator
pub static DISTANCE_CALCULATOR: &RustyDistanceCalculator = &RustyDistanceCalculator {};

/// Blockchain Module
pub struct BlockchainModule {
    /// Router sender
    pub router_sender: Sender<RouterThreadMessage<DursMsg>>,
    ///Path to the user datas profile
    pub profile_path: PathBuf,
    /// Currency
    pub currency: Option<CurrencyName>,
    /// Database
    pub db: Option<Db>,
    /// Fork tree
    pub fork_tree: ForkTree,
    /// Wot index
    pub wot_index: HashMap<PubKey, WotId>,
    /// Wots Databases
    pub wot_databases: WotsV10DBs,
    /// Currency parameters
    pub currency_params: Option<CurrencyParameters>,
    /// Current blockstamp
    pub current_blockstamp: Blockstamp,
    /// network consensus blockstamp
    pub consensus: Blockstamp,
    /// The block under construction
    pub pending_block: Option<Box<BlockDocument>>,
    /// Memorization of fork whose application fails
    pub invalid_forks: HashSet<Blockstamp>,
    /// pending network requests
    pub pending_network_requests: HashMap<ModuleReqId, OldNetworkRequest>,
    /// Last request blocks
    pub last_request_blocks: SystemTime,
    /// Last request fork blocks (=all blocks in fork window size)
    last_request_fork_blocks: SystemTime,
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

impl BlockchainModule {
    /// Return module identifier
    pub fn name() -> ModuleStaticName {
        ModuleStaticName(MODULE_NAME)
    }
    /// Loading blockchain configuration
    pub fn load_blockchain_conf(
        db: Db,
        router_sender: Sender<RouterThreadMessage<DursMsg>>,
        profile_path: PathBuf,
        _keys: RequiredKeysContent,
    ) -> BlockchainModule {
        // Get db path
        let dbs_path = durs_conf::get_blockchain_db_path(profile_path.clone());

        // Open databases
        let fork_tree = durs_bc_db_reader::current_meta_datas::get_fork_tree(&db)
            .unwrap_or_else(|_| fatal_error!("Fail to get fork tree."));
        let wot_databases = WotsV10DBs::open(Some(&dbs_path));

        // Get current blockstamp
        let current_blockstamp = durs_bc_db_reader::current_meta_datas::get_current_blockstamp(&db)
            .expect("Fatal error : fail to read Blockchain DB !")
            .unwrap_or_default();

        // Get currency parameters
        let (currency_name, currency_params) = if let Some((currency_name, currency_params)) =
            dubp_currency_params::db::get_currency_params(durs_conf::get_datas_path(
                profile_path.clone(),
            ))
            .expect("Fatal error : fail to read Blockchain DB !")
        {
            (Some(currency_name), Some(currency_params))
        } else {
            (None, None)
        };

        // Get wot index
        let wot_index: HashMap<PubKey, WotId> =
            durs_bc_db_reader::indexes::identities::get_wot_index(&db)
                .expect("Fatal eror : get_wot_index : Fail to read blockchain databases");

        // Instanciate BlockchainModule
        BlockchainModule {
            router_sender,
            profile_path,
            currency: currency_name,
            currency_params,
            current_blockstamp,
            consensus: Blockstamp::default(),
            db: Some(db),
            fork_tree,
            wot_index,
            wot_databases,
            pending_block: None,
            invalid_forks: HashSet::new(),
            pending_network_requests: HashMap::new(),
            last_request_blocks: UNIX_EPOCH,
            last_request_fork_blocks: UNIX_EPOCH,
        }
    }
    /// Databases explorer
    pub fn dbex(profile_path: PathBuf, csv: bool, req: &DbExQuery) {
        dbex::dbex(profile_path, csv, req);
    }
    /// Synchronize blockchain from local duniter json files
    pub fn local_sync<DC: DursConfTrait>(
        conf: &DC,
        currency_name: Option<&CurrencyName>,
        profile_path: PathBuf,
        sync_opts: SyncOpt,
    ) -> Result<(), Error> {
        Ok(sync::local_sync(
            conf,
            currency_name,
            profile_path,
            sync_opts,
        )?)
    }
    /// Start blockchain module.
    pub fn start_blockchain(
        &mut self,
        blockchain_receiver: &Receiver<DursMsg>,
        sync_opts: Option<SyncOpt>,
    ) {
        info!("BlockchainModule::start_blockchain()");

        // Send currency parameters to other modules
        if let Some(currency_params) = self.currency_params {
            events::sent::send_event(self, &BlockchainEvent::CurrencyParameters(currency_params));
        }

        if let Some(_sync_opts) = sync_opts {
            // TODO ...
        } else {
            // Start main loop
            self.main_loop(blockchain_receiver);
        }
    }
    /// Take blockchain database
    #[inline]
    pub fn take_db(&mut self) -> Db {
        self.db
            .take()
            .unwrap_or_else(|| fatal_error!("Dev error: none bc db."))
    }
    /// Reference to blockchain database
    #[inline]
    pub fn db(&self) -> &Db {
        if let Some(ref db) = self.db {
            db
        } else {
            fatal_error!("Dev error: none bc db.")
        }
    }

    /// Start blockchain main loop
    pub fn main_loop(&mut self, blockchain_receiver: &Receiver<DursMsg>) {
        // Init main loop datas
        let mut last_get_stackables_blocks = UNIX_EPOCH;

        loop {
            let now = SystemTime::now();
            // Request Consensus
            requests::sent::request_network_consensus(self);
            // Request next main blocks
            requests::sent::request_next_main_blocks(self, now);
            // Request fork blocks
            requests::sent::request_fork_blocks(self, now);

            // Listen received messages
            match blockchain_receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(durs_message) => {
                    match durs_message {
                        DursMsg::Request {
                            req_from,
                            req_id,
                            req_content,
                            ..
                        } => {
                            requests::received::receive_req(self, req_from, req_id, req_content);
                        }
                        DursMsg::Event {
                            event_type,
                            event_content,
                            ..
                        } => events::received::receive_event(self, event_type, event_content),
                        DursMsg::Response {
                            req_id,
                            res_content,
                            ..
                        } => responses::received::receive_response(self, req_id, res_content),
                        DursMsg::Stop => break,
                        _ => {} // Others DursMsg variants
                    }
                }
                Err(e) => match e {
                    RecvTimeoutError::Disconnected => {
                        fatal_error!("Disconnected blockchain module !");
                    }
                    RecvTimeoutError::Timeout => {}
                },
            }
            // Try to apply local stackable blocks every 20 seconds
            let now = SystemTime::now();
            if now
                .duration_since(last_get_stackables_blocks)
                .expect("duration_since error")
                > Duration::new(20, 0)
            {
                last_get_stackables_blocks = now;
                fork::stackable_blocks::apply_stackable_blocks(self);
                // Print current_blockstamp
                info!(
                    "BlockchainModule : current_blockstamp() = {:?}",
                    self.current_blockstamp
                );
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use tempfile::tempdir;

    #[inline]
    /// Open database in an arbitrary temporary directory given by OS
    /// and automatically cleaned when `Db` is dropped
    pub fn open_tmp_db() -> Result<Db, DbError> {
        open_db(tempdir().map_err(DbError::FileSystemError)?.path())
    }
}

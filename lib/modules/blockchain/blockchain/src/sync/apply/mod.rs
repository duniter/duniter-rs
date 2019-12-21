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

pub mod blocks_worker;
pub mod txs_worker;
pub mod wot_worker;

use crate::dubp;
use crate::dubp::apply::apply_valid_block;
use crate::dubp::apply::{ApplyValidBlockError, WriteBlockQueries};
use crate::sync::SyncJobsMess;
use crate::Db;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockNumber, Blockstamp};
use dubp_currency_params::{CurrencyName, CurrencyParameters};
use dup_crypto::keys::PubKey;
use durs_bc_db_reader::BcDbRead;
use durs_bc_db_writer::writers::requests::WotsDBsWriteQuery;
use durs_bc_db_writer::WotsV10DBs;
use durs_common_tools::fatal_error;
use durs_network_documents::url::Url;
use durs_wot::data::rusty::RustyWebOfTrust;
use durs_wot::data::WotId;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use unwrap::unwrap;

// récupérer les métadonnées entre deux utilisation
pub struct BlockApplicator {
    // options
    pub source: Option<Url>,
    pub currency: CurrencyName,
    pub verif_inner_hash: bool,
    pub currency_params: Option<CurrencyParameters>,
    pub dbs_path: PathBuf,
    pub target_blockstamp: Blockstamp,
    // senders
    pub sender_blocks_thread: mpsc::Sender<SyncJobsMess>,
    pub sender_wot_thread: mpsc::Sender<SyncJobsMess>,
    pub sender_tx_thread: mpsc::Sender<SyncJobsMess>,
    // pool
    pub certs_count: i32,
    pub current_blockstamp: Blockstamp,
    pub blocks_not_expiring: VecDeque<u64>,
    pub last_block_expiring: isize,
    // databases
    pub db: Option<Db>,
    pub wot_index: HashMap<PubKey, WotId>,
    pub wot_databases: WotsV10DBs,
    // time measurement
    pub wait_begin: SystemTime,
    pub all_wait_duration: Duration,
    pub all_verif_block_hashs_duration: Duration,
    pub all_apply_valid_block_duration: Duration,
}

impl BlockApplicator {
    pub fn apply(&mut self, block_doc: BlockDocument) {
        self.all_wait_duration += SystemTime::now().duration_since(self.wait_begin).unwrap();

        // Verify block hashs
        let verif_block_hashs_begin = SystemTime::now();
        if self.verif_inner_hash {
            dubp::check::hashs::check_block_hashes(&block_doc)
                .expect("Receive wrong block, please reset data and resync !");
        }
        self.all_verif_block_hashs_duration += SystemTime::now()
            .duration_since(verif_block_hashs_begin)
            .unwrap();

        // Push block common_time in blocks_not_expiring
        self.blocks_not_expiring.push_back(block_doc.common_time());
        // Get blocks_expiring
        let mut blocks_expiring = Vec::new();
        while self.blocks_not_expiring.front().cloned()
            < Some(block_doc.common_time() - unwrap!(self.currency_params).sig_validity)
        {
            self.last_block_expiring += 1;
            blocks_expiring.push(BlockNumber(self.last_block_expiring as u32));
            self.blocks_not_expiring.pop_front();
        }

        // Find expire_certs
        let expire_certs = if let Some(db) = self.db.take() {
            let expire_certs = db
                .r(|db_r| {
                    durs_bc_db_reader::indexes::certs::find_expire_certs(db_r, blocks_expiring)
                })
                .expect("find_expire_certs() : DbError");
            self.db = Some(db);
            expire_certs
        } else {
            fatal_error!("Dev error: BlockApplicator must have DB.")
        };

        // Get block blockstamp
        let blockstamp = block_doc.blockstamp();

        // Apply block
        let apply_valid_block_begin = SystemTime::now();
        let mut apply_valid_block_result = Err(ApplyValidBlockError::DBsCorrupted);
        if let Some(db) = self.db.take() {
            db.write(|mut w| {
                apply_valid_block_result = apply_valid_block::<RustyWebOfTrust>(
                    &db,
                    &mut w,
                    block_doc,
                    &mut self.wot_index,
                    &self.wot_databases.wot_db,
                    &expire_certs,
                );
                Ok(w)
            })
            .expect("Fail to apply valid block.");
            self.db = Some(db);
        } else {
            fatal_error!("Dev error: BlockApplicator must have DB.")
        }
        if let Ok(WriteBlockQueries(block_req, wot_db_reqs, currency_db_reqs)) =
            apply_valid_block_result
        {
            self.all_apply_valid_block_duration += SystemTime::now()
                .duration_since(apply_valid_block_begin)
                .unwrap();
            self.current_blockstamp = blockstamp;
            debug!("Apply db requests...");
            // Send block request to blocks worker thread
            self.sender_blocks_thread
                .send(SyncJobsMess::BlocksDBsWriteQuery(block_req))
                .expect(
                    "Fail to communicate with blocks worker thread, please reset data & resync !",
                );
            // Send wot requests to wot worker thread
            for req in wot_db_reqs {
                if let WotsDBsWriteQuery::CreateCert(
                    ref _source_pubkey,
                    ref _source,
                    ref _target,
                    ref _created_block_id,
                    ref _median_time,
                ) = req
                {
                    self.certs_count += 1;
                }
                self.sender_wot_thread
                    .send(SyncJobsMess::WotsDBsWriteQuery(
                        self.current_blockstamp,
                        Box::new(unwrap!(self.currency_params)),
                        req.clone(),
                    ))
                    .expect(
                        "Fail to communicate with tx worker thread, please reset data & resync !",
                    )
            }

            // In fork window ?
            let fork_window_size = unwrap!(self.currency_params).fork_window_size as u32;
            let in_fork_window = self.target_blockstamp.id.0 < fork_window_size
                || self.current_blockstamp.id.0 > self.target_blockstamp.id.0 - fork_window_size;

            // Send blocks and wot requests to wot worker thread
            for req in currency_db_reqs {
                self.sender_tx_thread
                    .send(SyncJobsMess::CurrencyDBsWriteQuery {
                        in_fork_window,
                        req: req.clone(),
                    })
                    .expect(
                        "Fail to communicate with tx worker thread, please reset data & resync !",
                    );
            }
            debug!("Success to apply block #{}", self.current_blockstamp.id.0);
            if self.current_blockstamp.id.0 >= self.target_blockstamp.id.0 {
                if self.current_blockstamp == self.target_blockstamp {
                    // Sync completed
                    return;
                } else {
                    fatal_error!("Fatal Error : we get a fork, please reset data and sync again !");
                }
            }
        } else {
            fatal_error!(
                "Fatal error : fail to stack up block #{}",
                self.current_blockstamp.id.0 + 1
            )
        }
        self.wait_begin = SystemTime::now();
    }
}

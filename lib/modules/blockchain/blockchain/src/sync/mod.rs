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

mod apply;
mod download;

use crate::dubp::apply::apply_valid_block;
use crate::*;
use dubp_documents::{BlockHash, BlockNumber};
use dup_crypto::keys::*;
use dup_currency_params::{CurrencyName, CurrencyParameters};
use durs_blockchain_dal::writers::requests::*;
use durs_common_tools::fatal_error;
use durs_wot::NodeId;
use pbr::ProgressBar;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use threadpool::ThreadPool;
use unwrap::unwrap;

/// Number of sync jobs
pub static NB_SYNC_JOBS: &'static usize = &4;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Block header
pub struct BlockHeader {
    pub number: BlockNumber,
    pub hash: BlockHash,
    pub issuer: PubKey,
}

#[derive(Debug, Clone)]
/// Message for main sync thread
pub enum MessForSyncThread {
    Target(CurrencyName, Blockstamp),
    BlockDocument(BlockDocument),
    DownloadFinish(),
    ApplyFinish(),
}

#[derive(Debug)]
/// Message for a job thread
pub enum SyncJobsMess {
    BlocksDBsWriteQuery(BlocksDBsWriteQuery),
    WotsDBsWriteQuery(Blockstamp, Box<CurrencyParameters>, WotsDBsWriteQuery),
    CurrencyDBsWriteQuery(Blockstamp, CurrencyDBsWriteQuery),
    End(),
}

/// Get json files path
fn get_json_files_path(source: Option<String>, currency: Option<String>) -> PathBuf {
    if let Some(ref path) = source {
        PathBuf::from(path)
    } else {
        let mut json_chunks_path = match dirs::config_dir() {
            Some(path) => path,
            None => fatal_error!("Impossible to get user config directory !"),
        };
        json_chunks_path.push("duniter/");
        json_chunks_path.push("duniter_default");

        let currency = if let Some(currency) = &currency {
            currency
        } else {
            DEFAULT_CURRENCY
        };

        json_chunks_path.push(currency);
        json_chunks_path
    }
}

/// Sync from local json files
pub fn local_sync<DC: DursConfTrait>(profile_path: PathBuf, conf: &DC, sync_opts: SyncOpt) {
    let SyncOpt {
        source,
        currency,
        end,
        cautious_mode: cautious,
        unsafe_mode: verif_inner_hash,
        ..
    } = sync_opts;

    // get json_files_path
    let json_files_path = get_json_files_path(source, currency);
    if !json_files_path.as_path().exists() {
        fatal_error!("duniter json chunks folder don't exist !");
    }

    // Get verification level
    let _verif_level = if cautious {
        println!("Start cautious sync...");
        info!("Start cautious sync...");
        SyncVerificationLevel::Cautious()
    } else {
        println!("Start fast sync...");
        info!("Start fast sync...");
        SyncVerificationLevel::FastSync()
    };

    // Create sync_thread channels
    let (sender_sync_thread, recv_sync_thread) = mpsc::channel();

    // Create ThreadPool
    let nb_cpus = num_cpus::get();
    let nb_workers = if nb_cpus < *NB_SYNC_JOBS {
        nb_cpus
    } else {
        *NB_SYNC_JOBS
    };
    let pool = ThreadPool::new(nb_workers);

    if !json_files_path.is_dir() {
        error!("json_files_path must be a directory");
        fatal_error!("json_files_path must be a directory");
    }

    // Lauch json reader worker
    download::json_reader_worker::json_reader_worker(
        &pool,
        profile_path.clone(),
        sender_sync_thread.clone(),
        json_files_path,
        end,
    );

    // Get target blockstamp
    let (currency, target_blockstamp) =
        if let Ok(MessForSyncThread::Target(currency, target_blockstamp)) = recv_sync_thread.recv()
        {
            (currency, target_blockstamp)
        } else {
            fatal_error!("Fatal error : no target blockstamp !");
        };

    // Update DursConf
    let mut conf = conf.clone();
    conf.set_currency(currency.clone());

    // Get databases path
    let db_path = durs_conf::get_blockchain_db_path(profile_path.clone());

    // Write new conf
    let mut conf_path = profile_path.clone();
    conf_path.push(durs_conf::constants::CONF_FILENAME);
    durs_conf::write_conf_file(conf_path.as_path(), &conf).expect("Fail to write new conf !");

    // Open blocks databases
    let blocks_dbs = BlocksV10DBs::open(Some(&db_path));

    // Open forks databases
    let forks_dbs = ForksDBs::open(Some(&db_path));

    // Open wot databases
    let wot_databases = WotsV10DBs::open(Some(&db_path));

    // Get local current blockstamp
    debug!("Get local current blockstamp...");
    let mut current_blockstamp: Blockstamp =
        durs_blockchain_dal::readers::block::get_current_blockstamp(&blocks_dbs)
            .expect("DALError : fail to get current blockstamp !")
            .unwrap_or_default();
    debug!("Success to get local current blockstamp.");

    // Node is already synchronized ?
    if target_blockstamp.id.0 <= current_blockstamp.id.0 {
        println!("Your durs node is already synchronized.");
        return;
    }

    // Get wot index
    let mut wot_index: HashMap<PubKey, NodeId> =
        readers::identity::get_wot_index(&wot_databases.identities_db)
            .expect("Fatal eror : get_wot_index : Fail to read blockchain databases");

    // Start sync
    let sync_start_time = SystemTime::now();

    // Count number of blocks and chunks
    let count_blocks = target_blockstamp.id.0 + 1 - current_blockstamp.id.0;
    let count_chunks = if count_blocks % 250 > 0 {
        (count_blocks / 250) + 1
    } else {
        count_blocks / 250
    };
    println!(
        "Sync from #{} to #{} :",
        current_blockstamp.id.0, target_blockstamp.id.0
    );
    info!(
        "Sync from #{} to #{} :",
        current_blockstamp.id.0, target_blockstamp.id.0
    );

    // Instantiate currency parameters
    let mut currency_params = None;

    // Createprogess bar
    let mut apply_pb = ProgressBar::new(count_chunks.into());
    apply_pb.format("╢▌▌░╟");

    // Create workers threads channels
    let (sender_blocks_thread, recv_blocks_thread) = mpsc::channel();
    let (sender_wot_thread, recv_wot_thread) = mpsc::channel();
    let (sender_tx_thread, recv_tx_thread) = mpsc::channel();

    // Launch blocks_worker thread
    apply::blocks_worker::execute(
        &pool,
        sender_sync_thread.clone(),
        recv_blocks_thread,
        blocks_dbs,
        forks_dbs,
        target_blockstamp,
        apply_pb,
    );

    // / Launch wot_worker thread
    apply::wot_worker::execute(
        &pool,
        profile_path.clone(),
        sender_sync_thread.clone(),
        recv_wot_thread,
    );

    // Launch tx_worker thread
    apply::txs_worker::execute(
        &pool,
        profile_path.clone(),
        sender_sync_thread.clone(),
        recv_tx_thread,
    );

    let main_job_begin = SystemTime::now();

    // Apply blocks
    let mut blocks_not_expiring = VecDeque::with_capacity(200_000);
    let mut last_block_expiring: isize = -1;
    let certs_db =
        BinDB::Mem(open_memory_db::<CertsExpirV10Datas>().expect("Fail to create memory certs_db"));
    let mut get_currency_params = false;
    let mut certs_count = 0;

    let mut all_wait_duration = Duration::from_millis(0);
    let mut wait_begin = SystemTime::now();
    let mut all_verif_block_hashs_duration = Duration::from_millis(0);
    let mut all_apply_valid_block_duration = Duration::from_millis(0);
    while let Ok(MessForSyncThread::BlockDocument(block_doc)) = recv_sync_thread.recv() {
        all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();

        // Verify block hashs
        let verif_block_hashs_begin = SystemTime::now();
        if verif_inner_hash {
            dubp::check::hashs::verify_block_hashs(&block_doc)
                .expect("Receive wrong block, please reset data and resync !");
        }
        all_verif_block_hashs_duration += SystemTime::now()
            .duration_since(verif_block_hashs_begin)
            .unwrap();
        // Get and write currency params
        if !get_currency_params {
            let datas_path = durs_conf::get_datas_path(profile_path.clone());
            if block_doc.number == BlockNumber(0) {
                currency_params = Some(
                    durs_blockchain_dal::readers::currency_params::get_and_write_currency_params(
                        &datas_path,
                        &block_doc,
                    ),
                );
            } else {
                currency_params = match dup_currency_params::db::get_currency_params(datas_path) {
                    Ok(Some((_currency_name, currency_params))) => Some(currency_params),
                    Ok(None) => fatal_error!("Params db corrupted: please reset data and resync !"),
                    Err(_) => fatal_error!("Fail to open params db"),
                }
            }
            get_currency_params = true;
        }
        let currency_params = unwrap!(currency_params);

        // Push block median_time in blocks_not_expiring
        blocks_not_expiring.push_back(block_doc.median_time);
        // Get blocks_expiring
        let mut blocks_expiring = Vec::new();
        while blocks_not_expiring.front().cloned()
            < Some(block_doc.median_time - currency_params.sig_validity)
        {
            last_block_expiring += 1;
            blocks_expiring.push(BlockNumber(last_block_expiring as u32));
            blocks_not_expiring.pop_front();
        }
        // Find expire_certs
        let expire_certs =
            durs_blockchain_dal::readers::certs::find_expire_certs(&certs_db, blocks_expiring)
                .expect("find_expire_certs() : DALError");
        // Get block blockstamp
        let blockstamp = block_doc.blockstamp();
        // Apply block
        let apply_valid_block_begin = SystemTime::now();
        if let Ok(ValidBlockApplyReqs(block_req, wot_db_reqs, currency_db_reqs)) =
            apply_valid_block::<RustyWebOfTrust>(
                block_doc,
                &mut wot_index,
                &wot_databases.wot_db,
                &expire_certs,
            )
        {
            all_apply_valid_block_duration += SystemTime::now()
                .duration_since(apply_valid_block_begin)
                .unwrap();
            current_blockstamp = blockstamp;
            debug!("Apply db requests...");
            // Send block request to blocks worker thread
            sender_blocks_thread
                .send(SyncJobsMess::BlocksDBsWriteQuery(block_req.clone()))
                .expect(
                    "Fail to communicate with blocks worker thread, please reset data & resync !",
                );
            // Send wot requests to wot worker thread
            for req in wot_db_reqs {
                if let WotsDBsWriteQuery::CreateCert(
                    ref _source_pubkey,
                    ref source,
                    ref target,
                    ref created_block_id,
                    ref _median_time,
                ) = req
                {
                    certs_count += 1;
                    // Add cert in certs_db
                    certs_db
                        .write(|db| {
                            let mut created_certs =
                                db.get(&created_block_id).cloned().unwrap_or_default();
                            created_certs.insert((*source, *target));
                            db.insert(*created_block_id, created_certs);
                        })
                        .expect("RustBreakError : please reset data and resync !");
                }
                sender_wot_thread
                    .send(SyncJobsMess::WotsDBsWriteQuery(
                        current_blockstamp,
                        Box::new(currency_params),
                        req.clone(),
                    ))
                    .expect(
                        "Fail to communicate with tx worker thread, please reset data & resync !",
                    )
            }
            // Send blocks and wot requests to wot worker thread
            for req in currency_db_reqs {
                sender_tx_thread
                    .send(SyncJobsMess::CurrencyDBsWriteQuery(
                        current_blockstamp,
                        req.clone(),
                    ))
                    .expect(
                        "Fail to communicate with tx worker thread, please reset data & resync !",
                    );
            }
            debug!("Success to apply block #{}", current_blockstamp.id.0);
            if current_blockstamp.id.0 >= target_blockstamp.id.0 {
                if current_blockstamp == target_blockstamp {
                    // Sync completed
                    break;
                } else {
                    fatal_error!("Fatal Error : we get a fork, please reset data and sync again !");
                }
            }
        } else {
            fatal_error!(
                "Fatal error : fail to stack up block #{}",
                current_blockstamp.id.0 + 1
            )
        }
        wait_begin = SystemTime::now();
    }
    // Send end signal to workers threads
    sender_blocks_thread
        .send(SyncJobsMess::End())
        .expect("Sync : Fail to send End signal to blocks worker !");
    info!("Sync : send End signal to blocks job.");
    sender_wot_thread
        .send(SyncJobsMess::End())
        .expect("Sync : Fail to send End signal to wot worker !");
    info!("Sync : send End signal to wot job.");
    sender_tx_thread
        .send(SyncJobsMess::End())
        .expect("Sync : Fail to send End signal to writer worker !");
    info!("Sync : send End signal to tx job.");

    // Save wot db
    wot_databases.wot_db.save().expect("Fail to save wot db");

    let main_job_duration =
        SystemTime::now().duration_since(main_job_begin).unwrap() - all_wait_duration;
    info!(
        "main_job_duration={},{:03} seconds.",
        main_job_duration.as_secs(),
        main_job_duration.subsec_millis()
    );
    info!(
        "all_verif_block_hashs_duration={},{:03} seconds.",
        all_verif_block_hashs_duration.as_secs(),
        all_verif_block_hashs_duration.subsec_millis()
    );
    info!(
        "all_apply_valid_block_duration={},{:03} seconds.",
        all_apply_valid_block_duration.as_secs(),
        all_apply_valid_block_duration.subsec_millis()
    );

    // Wait recv two finish signals
    let mut wait_jobs = *NB_SYNC_JOBS - 1;
    while wait_jobs > 0 {
        match recv_sync_thread.recv() {
            Ok(MessForSyncThread::ApplyFinish()) => wait_jobs -= 1,
            Ok(_) => thread::sleep(Duration::from_millis(50)),
            Err(_) => wait_jobs -= 1,
        }
    }
    info!("All sync jobs finish.");

    // Log sync duration
    debug!("certs_count={}", certs_count);
    let sync_duration = SystemTime::now().duration_since(sync_start_time).unwrap();
    println!(
        "Sync {} blocks in {}.{:03} seconds.",
        count_blocks,
        sync_duration.as_secs(),
        sync_duration.subsec_millis(),
    );
    info!(
        "Sync {} blocks in {}.{:03} seconds.",
        count_blocks,
        sync_duration.as_secs(),
        sync_duration.subsec_millis(),
    );
}

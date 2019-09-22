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

mod apply;
mod download;

use crate::*;
use apply::BlockApplicator;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::Blockstamp;
use dubp_common_doc::{BlockHash, BlockNumber};
use dubp_currency_params::{CurrencyName, CurrencyParameters};
use dup_crypto::keys::*;
use durs_bc_db_reader::CertsExpirV10Datas;
use durs_bc_db_writer::open_free_struct_memory_db;
use durs_bc_db_writer::writers::requests::*;
use durs_common_tools::fatal_error;
use durs_wot::WotId;
use failure::Fail;
use pbr::ProgressBar;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::time::SystemTime;
use std::{fs, thread};
use threadpool::ThreadPool;
use unwrap::unwrap;

/// Number of sync jobs
pub static NB_SYNC_JOBS: &usize = &4;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Block header
pub struct BlockHeader {
    pub number: BlockNumber,
    pub hash: BlockHash,
    pub issuer: PubKey,
}

#[derive(Debug)]
/// Message for main sync thread
pub enum MessForSyncThread {
    Target(CurrencyName, Blockstamp),
    BlockDocument(BlockDocument),
    DownloadFinish(),
    ApplyFinish(Option<Db>),
}

#[derive(Debug)]
/// Message for a job thread
pub enum SyncJobsMess {
    ForkWindowSize(usize), // informs block worker of fork window size
    BlocksDBsWriteQuery(BlocksDBsWriteQuery),
    WotsDBsWriteQuery(Blockstamp, Box<CurrencyParameters>, WotsDBsWriteQuery),
    CurrencyDBsWriteQuery {
        in_fork_window: bool,
        req: CurrencyDBsWriteQuery,
    },
    End,
}

#[derive(Clone, Debug, Fail)]
/// Local sync error
pub enum LocalSyncError {
    /// Fail to open database
    #[fail(
        display = "Unable to open the database, it may be a problem of access rights to the folder"
    )]
    FailToOpenDB,
    #[fail(
        display = "The folder you specified contains the blockchain of currency {}, \
        and your node already contains the blockchain of another currency {}. If you \
        wish to change currency you must reset your data ('reset data' command) or use a different profile (-p option).",
        found, expected
    )]
    /// Target currency and local currency are different
    InvalidTargetCurrency {
        expected: CurrencyName,
        found: CurrencyName,
    },
}

/// Sync from local json files
pub fn local_sync<DC: DursConfTrait>(
    conf: &DC,
    currency: Option<&CurrencyName>,
    profile_path: PathBuf,
    sync_opts: SyncOpt,
) -> Result<(), LocalSyncError> {
    let SyncOpt {
        cautious_mode: cautious,
        end,
        local_path,
        source,
        unsafe_mode,
        ..
    } = sync_opts;

    // get json_files_path
    let json_files_path = unwrap!(local_path);
    if !json_files_path.as_path().exists() {
        fatal_error!("duniter json chunks folder don't exist !");
    }

    // Get verification level
    let _verif_level = if cautious {
        info!("Start cautious sync...");
        SyncVerificationLevel::Cautious()
    } else {
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

    // Get target blockstamp and target currency
    let (target_currency, target_blockstamp) =
        if let Ok(MessForSyncThread::Target(target_currency, target_blockstamp)) =
            recv_sync_thread.recv()
        {
            (target_currency, target_blockstamp)
        } else {
            fatal_error!("Fatal error : no target blockstamp !");
        };

    // Check the consistency between currency and target_currency
    let currency = if let Some(currency) = currency {
        if currency == &target_currency {
            target_currency
        } else {
            return Err(LocalSyncError::InvalidTargetCurrency {
                expected: currency.clone(),
                found: target_currency,
            });
        }
    } else {
        target_currency
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

    // Open database
    let db = open_db(&db_path.as_path()).map_err(|_| LocalSyncError::FailToOpenDB)?;

    // Open wot databases
    let wot_databases = WotsV10DBs::open(Some(&db_path));

    // Get local current blockstamp
    debug!("Get local current blockstamp...");
    let current_blockstamp: Blockstamp =
        durs_bc_db_reader::current_meta_datas::get_current_blockstamp(&db)
            .expect("DbError : fail to get current blockstamp !")
            .unwrap_or_default();
    debug!("Success to get local current blockstamp.");

    // Node is already synchronized ?
    if target_blockstamp.id.0 <= current_blockstamp.id.0 {
        println!("Your durs node is already synchronized.");
        return Ok(());
    }

    // Get wot index
    let wot_index: HashMap<PubKey, WotId> =
        durs_bc_db_reader::indexes::identities::get_wot_index(&db)
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
        db,
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

    // Open databases
    let dbs_path = durs_conf::get_blockchain_db_path(profile_path.clone());
    let db = open_db(dbs_path.as_path()).expect("Fail to open blockchain DB.");
    let certs_db = BinFreeStructDb::Mem(
        open_free_struct_memory_db::<CertsExpirV10Datas>().expect("Fail to create memory certs_db"),
    );

    // initialise le BlockApplicator
    let mut block_applicator = BlockApplicator {
        source,
        currency,
        currency_params: None,
        dbs_path,
        db: Some(db),
        verif_inner_hash: !unsafe_mode,
        target_blockstamp,
        current_blockstamp,
        sender_blocks_thread,
        sender_wot_thread,
        sender_tx_thread,
        wot_index,
        wot_databases,
        certs_count: 0,
        blocks_not_expiring: VecDeque::with_capacity(200_000),
        last_block_expiring: -1,
        certs_db,
        wait_begin: SystemTime::now(),
        all_wait_duration: Duration::from_millis(0),
        all_verif_block_hashs_duration: Duration::from_millis(0),
        all_apply_valid_block_duration: Duration::from_millis(0),
    };

    // main loop
    let mut got_currency_params = false;
    while let Ok(MessForSyncThread::BlockDocument(block_doc)) = recv_sync_thread.recv() {
        // Get and write currency params
        if !got_currency_params {
            let datas_path = durs_conf::get_datas_path(profile_path.clone());
            if block_doc.number() == BlockNumber(0) {
                block_applicator.currency_params = Some(
                    durs_bc_db_reader::currency_params::get_and_write_currency_params(
                        &datas_path,
                        &block_doc,
                    ),
                );
            } else {
                block_applicator.currency_params =
                    match dubp_currency_params::db::get_currency_params(datas_path) {
                        Ok(Some((_currency_name, currency_params))) => Some(currency_params),
                        Ok(None) => {
                            fatal_error!("Params db corrupted: please reset data and resync !")
                        }
                        Err(_) => fatal_error!("Fail to open params db"),
                    }
            }
            got_currency_params = true;

            // Sends fork_window_size to block worker
            if block_applicator
                .sender_blocks_thread
                .send(SyncJobsMess::ForkWindowSize(
                    unwrap!(block_applicator.currency_params).fork_window_size,
                ))
                .is_err()
            {
                fatal_error!("Fail to communicate with blocks worker thread!");
            }
        }

        block_applicator.apply(block_doc);
    }

    // Send end signal to workers threads
    block_applicator
        .sender_blocks_thread
        .send(SyncJobsMess::End)
        .expect("Sync : Fail to send End signal to blocks worker !");
    info!("Sync : send End signal to blocks job.");
    block_applicator
        .sender_wot_thread
        .send(SyncJobsMess::End)
        .expect("Sync : Fail to send End signal to wot worker !");
    info!("Sync : send End signal to wot job.");
    block_applicator
        .sender_tx_thread
        .send(SyncJobsMess::End)
        .expect("Sync : Fail to send End signal to writer worker !");
    info!("Sync : send End signal to tx job.");

    // Save wot db
    block_applicator
        .wot_databases
        .wot_db
        .save()
        .expect("Fail to save wot db");

    let main_job_duration = SystemTime::now().duration_since(main_job_begin).unwrap()
        - block_applicator.all_wait_duration;
    info!(
        "main_job_duration={},{:03} seconds.",
        main_job_duration.as_secs(),
        main_job_duration.subsec_millis()
    );
    info!(
        "all_verif_block_hashs_duration={},{:03} seconds.",
        block_applicator.all_verif_block_hashs_duration.as_secs(),
        block_applicator
            .all_verif_block_hashs_duration
            .subsec_millis()
    );
    info!(
        "all_apply_valid_block_duration={},{:03} seconds.",
        block_applicator.all_apply_valid_block_duration.as_secs(),
        block_applicator
            .all_apply_valid_block_duration
            .subsec_millis()
    );

    // Wait recv two finish signals
    let mut wait_jobs = *NB_SYNC_JOBS - 1;
    let mut db = None;
    while wait_jobs > 0 {
        match recv_sync_thread.recv() {
            Ok(MessForSyncThread::ApplyFinish(db_opt)) => {
                if db.is_none() {
                    db = db_opt;
                }
                wait_jobs -= 1;
            }
            Ok(_) => thread::sleep(Duration::from_millis(50)),
            Err(_) => wait_jobs -= 1,
        }
    }
    info!("All sync jobs finish.");

    // Save blockchain DB
    if let Some(db) = db {
        db.save()
            .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));
    } else {
        fatal_error!("Dev error: sync workers didn't return the DB.")
    }

    // Log sync duration
    debug!("certs_count={}", block_applicator.certs_count);
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
    Ok(())
}

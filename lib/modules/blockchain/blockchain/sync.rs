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

extern crate num_cpus;
extern crate pbr;
extern crate sqlite;
extern crate threadpool;

use self::pbr::ProgressBar;
use self::threadpool::ThreadPool;
use dubp_documents::{BlockHash, BlockId};
use duniter_network::documents::NetworkBlock;
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_blockchain_dal::currency_params::CurrencyParameters;
use durs_blockchain_dal::writers::requests::*;
use durs_blockchain_dal::ForkId;
use durs_wot::NodeId;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use ts_parsers::*;
use *;

/// Number of sync jobs
pub static NB_SYNC_JOBS: &'static usize = &4;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Block header
pub struct BlockHeader {
    pub number: BlockId,
    pub hash: BlockHash,
    pub issuer: PubKey,
}

#[derive(Debug)]
/// Message for main sync thread
enum MessForSyncThread {
    Target(CurrencyName, Blockstamp),
    NetworkBlock(NetworkBlock),
    DownloadFinish(),
    ApplyFinish(),
}

#[derive(Debug)]
/// Message for a job thread
enum SyncJobsMess {
    BlocksDBsWriteQuery(BlocksDBsWriteQuery),
    WotsDBsWriteQuery(WotsDBsWriteQuery, Box<CurrencyParameters>),
    CurrencyDBsWriteQuery(CurrencyDBsWriteQuery),
    End(),
}

/// Sync from a duniter-ts database
pub fn sync_ts<DC: DuniterConf>(
    profile: &str,
    conf: &DC,
    db_ts_path: PathBuf,
    cautious: bool,
    verif_inner_hash: bool,
) {
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

    // Determine db_ts_copy_path
    let mut db_ts_copy_path = duniter_conf::datas_path(profile, &conf.currency().clone());
    db_ts_copy_path.push("tmp_db_ts_copy.db");

    // Lauch ts thread
    let profile_copy = String::from(profile);
    let sender_sync_thread_clone = sender_sync_thread.clone();
    pool.execute(move || {
        let ts_job_begin = SystemTime::now();
        // copy db_ts
        fs::copy(db_ts_path.as_path(), db_ts_copy_path.as_path())
            .expect("Fatal error : fail to copy duniter-ts database !");
        // open copy of db_ts
        let ts_db = sqlite::open(db_ts_copy_path.as_path())
            .expect("Fatal error : fail to open copy of duniter-ts database !");
        info!("sync_ts : Success to open duniter-ts database.");

        // Get ts current blockstamp
        debug!("Get ts-db current blockstamp...");
        let mut cursor: sqlite::Cursor = ts_db
            .prepare("SELECT hash, number, currency FROM block WHERE fork=? ORDER BY number DESC LIMIT 1;")
            .expect("Request SQL get_ts_current_block is wrong !")
            .cursor();
        cursor
            .bind(&[sqlite::Value::Integer(0)])
            .expect("Fail to get ts current block !");
        let (currency, current_ts_blockstamp) =
            if let Some(row) = cursor.next().expect("cursor error") {
                let block_id = BlockId(
                    row[1]
                        .as_integer()
                        .expect("Fail to parse current ts blockstamp !") as u32,
                );
                let block_hash = BlockHash(
                    Hash::from_hex(
                        row[0]
                            .as_string()
                            .expect("Fail to parse current ts blockstamp !"),
                    ).expect("Fail to parse current ts blockstamp !"),
                );
                (
                    CurrencyName(String::from(
                        row[2]
                            .as_string()
                            .expect("Fatal error :Fail to get currency !"),
                    )),
                    Blockstamp {
                        id: block_id,
                        hash: block_hash,
                    },
                )
            } else {
                panic!("Fail to get current ts blockstamp !");
            };

        debug!("Success to ts-db current blockstamp.");

        // Get current local blockstamp
        debug!("Get local current blockstamp...");
        let db_path = duniter_conf::get_blockchain_db_path(&profile_copy, &currency);
        let blocks_databases = BlocksV10DBs::open(Some(&db_path));
        let current_blockstamp: Blockstamp = durs_blockchain_dal::block::get_current_blockstamp(
            &blocks_databases,
        ).expect("ForksV10DB : RustBreakError !")
            .unwrap_or_default();
        debug!("Success to get local current blockstamp.");

        // Send ts current blockstamp
        sender_sync_thread_clone
            .send(MessForSyncThread::Target(
                currency.clone(),
                current_ts_blockstamp,
            ))
            .expect("Fatal error : sync_thread unrechable !");

        // Get genesis block
        if current_blockstamp == Blockstamp::default() {
            let mut cursor: sqlite::Cursor = ts_db
                    .prepare(
                        "SELECT hash, inner_hash, signature, currency, issuer, parameters, previousHash,
                            previousIssuer, version, membersCount, monetaryMass, medianTime, dividend, unitbase,
                            time, powMin, number, nonce, transactions, certifications, identities, joiners,
                            actives, leavers, revoked, excluded, issuersFrame, issuersFrameVar, issuersCount
                            FROM block WHERE fork=0 AND number=? LIMIT 1;",
                    )
                    .expect("Request SQL get_ts_blocks is wrong !")
                    .cursor();
            cursor
                .bind(&[sqlite::Value::Integer(0)])
                .expect("Fail to get genesis block !");
            if let Some(row) = cursor.next().expect("cursor error") {
                sender_sync_thread_clone
                    .send(MessForSyncThread::NetworkBlock(parse_ts_block(row)))
                    .expect("Fatal error : sync_thread unrechable !");
            }
        }

        // Request ts blocks
        let mut cursor: sqlite::Cursor = ts_db
                .prepare(
                    "SELECT hash, inner_hash, signature, currency, issuer, parameters, previousHash,
                        previousIssuer, version, membersCount, monetaryMass, medianTime, dividend, unitbase,
                        time, powMin, number, nonce, transactions, certifications, identities, joiners,
                        actives, leavers, revoked, excluded, issuersFrame, issuersFrameVar, issuersCount
                        FROM block WHERE fork=? AND number > ? ORDER BY number ASC;",
                )
                .expect("Request SQL get_ts_blocks is wrong !")
                .cursor();
        cursor
            .bind(&[
                sqlite::Value::Integer(0),
                sqlite::Value::Integer(i64::from(current_blockstamp.id.0)),
            ])
            .expect("0");

        // Parse ts blocks
        //let mut ts_blocks = Vec::with_capacity(current_ts_blockstamp.id.0 + 1);
        //let pool = ThreadPool::new(4);
        while let Some(row) = cursor.next().expect("cursor error") {
            //let sender_sync_thread_clone = sender_sync_thread.clone();
            //pool.execute(move || {
            sender_sync_thread_clone
                .send(MessForSyncThread::NetworkBlock(parse_ts_block(row)))
                .expect("Fatal error : sync_thread unrechable !");
            //});
        }
        fs::remove_file(db_ts_copy_path.as_path())
            .expect("Fatal error : fail to remove db_ts_copy !");
        sender_sync_thread_clone
            .send(MessForSyncThread::DownloadFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let ts_job_duration = SystemTime::now()
            .duration_since(ts_job_begin)
            .expect("duration_since error");
        info!(
            "ts_job_duration={},{:03} seconds.",
            ts_job_duration.as_secs(),
            ts_job_duration.subsec_millis()
        );
    });

    // Get currency and target blockstamp
    let (currency, target_blockstamp) =
        if let Ok(MessForSyncThread::Target(currency, target_blockstamp)) = recv_sync_thread.recv()
        {
            (currency, target_blockstamp)
        } else {
            panic!("Fatal error : no TargetBlockstamp !")
        };

    // Update DuniterConf
    let mut conf = conf.clone();
    conf.set_currency(currency.clone());

    // Get databases path
    let db_path = duniter_conf::get_blockchain_db_path(profile, &currency);

    // Write nex conf
    duniter_conf::write_conf_file(profile, &conf).expect("Fail to write new conf !");

    // Open wot db
    let wot_db = open_wot_db::<RustyWebOfTrust>(Some(&db_path)).expect("Fail to open WotDB !");

    // Open blocks databases
    let databases = BlocksV10DBs::open(Some(&db_path));

    // Open wot databases
    let wot_databases = WotsV10DBs::open(Some(&db_path));

    // Get local current blockstamp
    debug!("Get local current blockstamp...");
    let mut current_blockstamp: Blockstamp =
        durs_blockchain_dal::block::get_current_blockstamp(&databases)
            .expect("ForksV10DB : RustBreakError !")
            .unwrap_or_default();
    debug!("Success to get local current blockstamp.");

    // Node is already synchronized ?
    if target_blockstamp.id.0 < current_blockstamp.id.0 {
        println!("Your duniter-rs node is already synchronized.");
        return;
    }

    // Get wot index
    let mut wot_index: HashMap<PubKey, NodeId> =
        DALIdentity::get_wot_index(&wot_databases.identities_db)
            .expect("Fatal eror : get_wot_index : Fail to read blockchain databases");

    // Start sync
    let sync_start_time = SystemTime::now();
    info!(
        "Sync from #{} to #{}...",
        current_blockstamp.id.0, target_blockstamp.id.0
    );
    println!(
        "Sync from #{} to #{}...",
        current_blockstamp.id.0, target_blockstamp.id.0
    );

    // Createprogess bar
    let count_blocks = target_blockstamp.id.0 + 1 - current_blockstamp.id.0;
    let count_chunks = if count_blocks % 250 > 0 {
        (count_blocks / 250) + 1
    } else {
        count_blocks / 250
    };
    let mut apply_pb = ProgressBar::new(count_chunks.into());
    apply_pb.format("╢▌▌░╟");
    // Create workers threads channels
    let (sender_blocks_thread, recv_blocks_thread) = mpsc::channel();
    let (sender_tx_thread, recv_tx_thread) = mpsc::channel();
    let (sender_wot_thread, recv_wot_thread) = mpsc::channel();

    // Launch blocks_worker thread
    let sender_sync_thread_clone = sender_sync_thread.clone();
    pool.execute(move || {
        let blocks_job_begin = SystemTime::now();

        // Listen db requets
        let mut chunk_index = 0;
        let mut blockchain_meta_datas = HashMap::new();
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::BlocksDBsWriteQuery(req)) = recv_blocks_thread.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            // Apply db request
            req.apply(&databases, true)
                .expect("Fatal error : Fail to apply DBWriteRequest !");
            if let BlocksDBsWriteQuery::WriteBlock(
                ref _dal_block,
                ref _old_fork_id,
                ref previous_blockstamp,
                ref previous_hash,
            ) = req
            {
                blockchain_meta_datas.insert(*previous_blockstamp, *previous_hash);
                chunk_index += 1;
                if chunk_index == 250 {
                    chunk_index = 0;
                    apply_pb.inc();
                }
            }
            wait_begin = SystemTime::now();
        }

        // Indexing blockchain meta datas
        info!("Indexing blockchain meta datas...");
        /*let blockchain_meta_datas: HashMap<PreviousBlockstamp, BlockHash> = databases
        .blockchain_db
        .read(|db| {
            let mut blockchain_meta_datas: HashMap<
                PreviousBlockstamp,
                BlockHash,
            > = HashMap::new();
            for dal_block in db.values() {
                let block_previous_hash = if dal_block.block.number.0 == 0 {
                    PreviousBlockstamp::default()
                } else {
                    PreviousBlockstamp {
                        id: BlockId(dal_block.block.number.0 - 1),
                        hash: BlockHash(dal_block.block.previous_hash),
                    }
                };
                blockchain_meta_datas
                    .insert(block_previous_hash, dal_block.block.expect("Try to get hash of an uncompleted or reduce block !"));
            }
            blockchain_meta_datas
        })
        .expect("Indexing blockchain meta datas : DALError");*/
        databases
            .forks_db
            .write(|db| {
                db.insert(ForkId(0), blockchain_meta_datas);
            })
            .expect("Indexing blockchain meta datas : DALError");

        // Increment progress bar (last chunk)
        apply_pb.inc();
        // Save blockchain, and fork databases
        info!("Save blockchain and forks databases in files...");
        databases.save_dbs();

        // Send finish signal
        sender_sync_thread_clone
            .send(MessForSyncThread::ApplyFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let blocks_job_duration =
            SystemTime::now().duration_since(blocks_job_begin).unwrap() - all_wait_duration;
        info!(
            "blocks_job_duration={},{:03} seconds.",
            blocks_job_duration.as_secs(),
            blocks_job_duration.subsec_millis()
        );
    });

    // / Launch wot_worker thread
    let profile_copy2 = String::from(profile);
    let currency_copy2 = currency.clone();
    let sender_sync_thread_clone2 = sender_sync_thread.clone();

    pool.execute(move || {
        let wot_job_begin = SystemTime::now();
        // Open databases
        let db_path = duniter_conf::get_blockchain_db_path(&profile_copy2, &currency_copy2);
        let databases = WotsV10DBs::open(Some(&db_path));

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(mess) = recv_wot_thread.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            match mess {
                SyncJobsMess::WotsDBsWriteQuery(req, currency_params) => req
                    .apply(&databases, &currency_params.deref())
                    .expect("Fatal error : Fail to apply DBWriteRequest !"),
                SyncJobsMess::End() => break,
                _ => {}
            }
            wait_begin = SystemTime::now();
        }
        // Save wots databases
        info!("Save wots databases in files...");
        databases.save_dbs();

        // Send finish signal
        sender_sync_thread_clone2
            .send(MessForSyncThread::ApplyFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let wot_job_duration =
            SystemTime::now().duration_since(wot_job_begin).unwrap() - all_wait_duration;
        info!(
            "wot_job_duration={},{:03} seconds.",
            wot_job_duration.as_secs(),
            wot_job_duration.subsec_millis()
        );
    });

    // Launch tx_worker thread
    let profile_copy = String::from(profile);
    let currency_copy = conf.currency().clone();
    let sender_sync_thread_clone = sender_sync_thread.clone();
    pool.execute(move || {
        let tx_job_begin = SystemTime::now();
        // Open databases
        let db_path = duniter_conf::get_blockchain_db_path(&profile_copy, &currency_copy);
        let databases = CurrencyV10DBs::open(Some(&db_path));

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::CurrencyDBsWriteQuery(req)) = recv_tx_thread.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            // Apply db request
            req.apply(&databases)
                .expect("Fatal error : Fail to apply DBWriteRequest !");
            wait_begin = SystemTime::now();
        }
        // Save tx, utxo, du and balances databases
        info!("Save tx and sources database in file...");
        databases.save_dbs(true, true);

        // Send finish signal
        sender_sync_thread_clone
            .send(MessForSyncThread::ApplyFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let tx_job_duration =
            SystemTime::now().duration_since(tx_job_begin).unwrap() - all_wait_duration;
        info!(
            "tx_job_duration={},{:03} seconds.",
            tx_job_duration.as_secs(),
            tx_job_duration.subsec_millis()
        );
    });
    let main_job_begin = SystemTime::now();

    // Open currency_params_db
    let dbs_path = duniter_conf::get_blockchain_db_path(profile, &conf.currency());
    let currency_params_db =
        open_db::<CurrencyParamsV10Datas>(&dbs_path, "params.db").expect("Fail to open params db");

    // Apply blocks
    let mut blocks_not_expiring = VecDeque::with_capacity(200_000);
    let mut last_block_expiring: isize = -1;
    let certs_db =
        BinDB::Mem(open_memory_db::<CertsExpirV10Datas>().expect("Fail to create memory certs_db"));
    let mut currency_params = CurrencyParameters::default();
    let mut get_currency_params = false;
    let mut certs_count = 0;

    let mut all_wait_duration = Duration::from_millis(0);
    let mut wait_begin = SystemTime::now();
    let mut all_complete_block_duration = Duration::from_millis(0);
    let mut all_apply_valid_block_duration = Duration::from_millis(0);
    while let Ok(MessForSyncThread::NetworkBlock(network_block)) = recv_sync_thread.recv() {
        all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
        // Complete block
        let complete_block_begin = SystemTime::now();
        let block_doc = complete_network_block(&network_block, verif_inner_hash)
            .expect("Receive wrong block, please reset data and resync !");
        all_complete_block_duration += SystemTime::now()
            .duration_since(complete_block_begin)
            .unwrap();
        // Get currency params
        if !get_currency_params && block_doc.number.0 == 0 {
            if block_doc.parameters.is_some() {
                currency_params_db
                    .write(|db| {
                        db.0 = block_doc.currency.clone();
                        db.1 = block_doc.parameters.unwrap();
                    })
                    .expect("fail to write in params DB");
                currency_params = CurrencyParameters::from((
                    block_doc.currency.clone(),
                    block_doc.parameters.unwrap(),
                ));
                get_currency_params = true;
            } else {
                panic!("The genesis block are None parameters !");
            }
        }
        // Push block median_time in blocks_not_expiring
        blocks_not_expiring.push_back(block_doc.median_time);
        // Get blocks_expiring
        let mut blocks_expiring = Vec::new();
        while blocks_not_expiring.front().cloned()
            < Some(block_doc.median_time - currency_params.sig_validity)
        {
            last_block_expiring += 1;
            blocks_expiring.push(BlockId(last_block_expiring as u32));
            blocks_not_expiring.pop_front();
        }
        // Find expire_certs
        let expire_certs =
            durs_blockchain_dal::certs::find_expire_certs(&certs_db, blocks_expiring)
                .expect("find_expire_certs() : DALError");
        // Apply block
        let apply_valid_block_begin = SystemTime::now();
        if let Ok(ValidBlockApplyReqs(block_req, wot_db_reqs, currency_db_reqs)) =
            apply_valid_block::<RustyWebOfTrust>(
                &block_doc,
                &mut wot_index,
                &wot_db,
                &expire_certs,
                None,
            )
        {
            all_apply_valid_block_duration += SystemTime::now()
                .duration_since(apply_valid_block_begin)
                .unwrap();
            current_blockstamp = network_block.blockstamp();
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
                        req.clone(),
                        Box::new(currency_params),
                    ))
                    .expect(
                        "Fail to communicate with tx worker thread, please reset data & resync !",
                    )
            }
            // Send blocks and wot requests to wot worker thread
            for req in currency_db_reqs {
                sender_tx_thread
                    .send(SyncJobsMess::CurrencyDBsWriteQuery(req.clone()))
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
                    panic!("Fatal Error : we get a fork, please reset data and sync again !");
                }
            }
        } else {
            panic!(
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

    // Save params db
    currency_params_db.save().expect("Fail to save params db");

    // Save wot file
    wot_db.save().expect("Fail to save wot db");

    let main_job_duration =
        SystemTime::now().duration_since(main_job_begin).unwrap() - all_wait_duration;
    info!(
        "main_job_duration={},{:03} seconds.",
        main_job_duration.as_secs(),
        main_job_duration.subsec_millis()
    );
    info!(
        "all_complete_block_duration={},{:03} seconds.",
        all_complete_block_duration.as_secs(),
        all_complete_block_duration.subsec_millis()
    );
    info!(
        "all_apply_valid_block_duration={},{:03} seconds.",
        all_apply_valid_block_duration.as_secs(),
        all_apply_valid_block_duration.subsec_millis()
    );

    // Wait recv two finish signals
    let mut wait_jobs = *NB_SYNC_JOBS - 1;
    while wait_jobs > 0 {
        if let Ok(MessForSyncThread::ApplyFinish()) = recv_sync_thread.recv() {
            wait_jobs -= 1;
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }
    info!("All sync jobs finish.");

    // Log sync duration
    println!("certs_count={}", certs_count);
    let sync_duration = SystemTime::now().duration_since(sync_start_time).unwrap();
    println!(
        "Sync {} blocks in {}.{:03} seconds.",
        current_blockstamp.id.0 + 1,
        sync_duration.as_secs(),
        sync_duration.subsec_millis(),
    );
    info!(
        "Sync {} blocks in {}.{:03} seconds.",
        current_blockstamp.id.0 + 1,
        sync_duration.as_secs(),
        sync_duration.subsec_millis(),
    );
}

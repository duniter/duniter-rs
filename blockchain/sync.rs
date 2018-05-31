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

extern crate duniter_conf;
extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_message;
extern crate duniter_module;
extern crate duniter_network;
extern crate pbr;
extern crate serde;
extern crate serde_json;
extern crate sqlite;

use self::pbr::ProgressBar;
use duniter_crypto::keys::*;
use duniter_dal::parsers::identities::parse_compact_identity;
use duniter_dal::parsers::transactions::parse_transaction;
//use duniter_dal::writers::requests::DBWriteRequest;
use duniter_documents::blockchain::v10::documents::membership::MembershipType;
use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::{BlockHash, BlockId, Hash};
use duniter_network::{NetworkBlock, NetworkBlockV10};
use duniter_wotb::{NodeId, WebOfTrust};
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub number: BlockId,
    pub hash: BlockHash,
    pub issuer: PubKey,
}

enum ParserWorkMess {
    TargetBlockstamp(Blockstamp),
    NetworkBlock(NetworkBlock),
    //DBWriteRequest(DBWriteRequest),
    End(),
}

pub fn sync_ts(
    conf: &DuniterConf,
    current_blockstamp: &Blockstamp,
    db_ts_path: PathBuf,
    cautious: bool,
) {
    // get profile and currency and current_blockstamp
    let profile = &conf.profile();
    let currency = &conf.currency();
    let mut current_blockstamp = *current_blockstamp;

    // Copy blockchain db in ramfs
    let db_path = duniter_conf::get_db_path(profile, currency, false);
    if db_path.as_path().exists() {
        info!("Copy blockchain DB in ramfs...");
        fs::copy(db_path, format!("/dev/shm/{}_durs.db", profile))
            .expect("Fatal error : fail to copy DB in ramfs !");
    }

    // Get wot path
    let wot_path = duniter_conf::get_wot_path(profile.clone().to_string(), currency);

    // Open wot file
    let (mut wot, mut _wot_blockstamp): (RustyWebOfTrust, Blockstamp) =
        if wot_path.as_path().exists() {
            match WOT_FILE_FORMATER.from_file(
                wot_path.as_path().to_str().unwrap(),
                duniter_dal::constants::G1_PARAMS.sig_stock as usize,
            ) {
                Ok((wot, binary_blockstamp)) => match str::from_utf8(&binary_blockstamp) {
                    Ok(str_blockstamp) => (wot, Blockstamp::from_string(str_blockstamp).unwrap()),
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                },
                Err(e) => panic!("Fatal Error : fail te read wot file : {:?}", e),
            }
        } else {
            (
                RustyWebOfTrust::new(duniter_dal::constants::G1_PARAMS.sig_stock as usize),
                Blockstamp::default(),
            )
        };

    // Get verification level
    let verif_level = if cautious {
        println!("Start cautious sync...");
        info!("Start cautious sync...");
        SyncVerificationLevel::Cautious()
    } else {
        println!("Start fast sync...");
        info!("Start fast sync...");
        SyncVerificationLevel::FastSync()
    };

    // Create sync_thread channel
    let (sender_sync_thread, recv_sync_thread) = mpsc::channel();

    // Lauch ts thread
    thread::spawn(move || {
        // open db_ts
        let ts_db = sqlite::open(db_ts_path.as_path())
            .expect("Fatal error : fail to open duniter-ts database !");
        info!("sync_ts : Success to open duniter-ts database.");

        // Get ts current blockstamp
        debug!("Get ts-db current blockstamp...");
        let mut cursor: sqlite::Cursor = ts_db
            .prepare("SELECT hash, number FROM block WHERE fork=? ORDER BY number DESC LIMIT 1;")
            .expect("Request SQL get_ts_current_block is wrong !")
            .cursor();
        cursor
            .bind(&[sqlite::Value::Integer(0)])
            .expect("Fail to get ts current block !");
        let current_ts_blockstamp = if let Some(row) = cursor.next().expect("cursor error") {
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
            Blockstamp {
                id: block_id,
                hash: block_hash,
            }
        } else {
            panic!("Fail to get current ts blockstamp !");
        };
        debug!("Success to ts-db current blockstamp.");

        // Send ts current blockstamp
        sender_sync_thread
            .send(ParserWorkMess::TargetBlockstamp(current_ts_blockstamp))
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
                sender_sync_thread
                    .send(ParserWorkMess::NetworkBlock(parse_ts_block(row)))
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
            sender_sync_thread
                .send(ParserWorkMess::NetworkBlock(parse_ts_block(row)))
                .expect("Fatal error : sync_thread unrechable !");
            //});
        }
        sender_sync_thread
            .send(ParserWorkMess::End())
            .expect("Fatal error : sync_thread unrechable !");
    });

    // Get target blockstamp
    let target_blockstamp =
        if let Ok(ParserWorkMess::TargetBlockstamp(target_blockstamp)) = recv_sync_thread.recv() {
            target_blockstamp
        } else {
            panic!("Fatal error : no TargetBlockstamp !")
        };

    // Instanciate blockchain module
    let blockchain_module =
        BlockchainModule::load_blockchain_conf(conf, RequiredKeysContent::None(), true);

    // Node is already synchronized ?
    if target_blockstamp.id.0 < current_blockstamp.id.0 {
        println!("Your duniter-rs node is already synchronized.");
        return;
    }

    // Get wotb index
    let mut wotb_index: HashMap<PubKey, NodeId> =
        DALIdentity::get_wotb_index(&blockchain_module.db);

    // Start sync
    let sync_start_time = SystemTime::now();
    println!(
        "Sync from #{} to #{} :",
        current_blockstamp.id.0, target_blockstamp.id.0
    );
    info!(
        "Sync from #{} to #{}...",
        current_blockstamp.id.0, target_blockstamp.id.0
    );
    let mut pb = ProgressBar::new((target_blockstamp.id.0 + 1 - current_blockstamp.id.0).into());

    // Apply blocks
    while let Ok(ParserWorkMess::NetworkBlock(network_block)) = recv_sync_thread.recv() {
        // Complete block
        let block_doc = complete_network_block(
            &blockchain_module.currency.to_string(),
            None,
            &network_block,
            SyncVerificationLevel::FastSync(),
        ).expect("Receive wrong block, please reset data and resync !");
        // Apply block
        let (success, db_requests, new_wot_events) =
            try_stack_up_completed_block::<RustyWebOfTrust>(&block_doc, &wotb_index, &wot);

        blockchain_module.try_stack_up_block::<RustyWebOfTrust>(
            &network_block,
            &wotb_index,
            &wot,
            verif_level,
        );
        if success {
            current_blockstamp = network_block.blockstamp();
            debug!("Apply db requests...");
            // Apply db requests
            db_requests
                .iter()
                .map(|req| req.apply(&conf.currency().to_string(), &blockchain_module.db))
                .collect::<()>();
            debug!("Finish to apply db requests.");
            // Apply WotEvents
            if !new_wot_events.is_empty() {
                for wot_event in new_wot_events {
                    match wot_event {
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
                if current_blockstamp.id.0 > target_blockstamp.id.0 - 100 {
                    // Save wot file
                    WOT_FILE_FORMATER
                        .to_file(
                            &wot,
                            current_blockstamp.to_string().as_bytes(),
                            wot_path.as_path().to_str().unwrap(),
                        )
                        .expect("Fatal Error: Fail to write wotb in file !");
                }
            }
            pb.inc();
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
    }

    // Copy memory db to real db
    info!("Save blockchain DB in profile folder...");
    fs::copy(
        format!("/dev/shm/{}_durs.db", profile),
        duniter_conf::get_db_path(profile, currency, false).as_path(),
    ).expect("Fatal error : fail to copy DB in profile folder !");

    // Remove memory db
    fs::remove_file(format!("/dev/shm/{}_durs.db", profile))
        .expect("Fatal error : fail to remove memory DB !");

    // Print sync duration
    let sync_duration = SystemTime::now().duration_since(sync_start_time).unwrap();
    println!(
        "Sync {} blocks in {}m {}s.",
        current_blockstamp.id.0,
        sync_duration.as_secs() / 60,
        sync_duration.as_secs() % 60,
    );
}

pub fn parse_ts_block(row: &[sqlite::Value]) -> NetworkBlock {
    // Parse block
    let current_header = BlockHeader {
        number: BlockId(row[16].as_integer().expect("Fail to parse block number") as u32),
        hash: BlockHash(
            Hash::from_hex(row[0].as_string().expect("Fail to parse block hash"))
                .expect("Fail to parse block hash (2)"),
        ),
        issuer: PubKey::Ed25519(
            ed25519::PublicKey::from_base58(
                row[4].as_string().expect("Fail to parse block issuer"),
            ).expect("Failt to parse block issuer (2)"),
        ),
    };
    let previous_header = if current_header.number.0 > 0 {
        Some(BlockHeader {
            number: BlockId(current_header.number.0 - 1),
            hash: BlockHash(
                Hash::from_hex(
                    row[6]
                        .as_string()
                        .expect("Fail to parse block previous hash"),
                ).expect("Fail to parse block previous hash (2)"),
            ),
            issuer: PubKey::Ed25519(
                ed25519::PublicKey::from_base58(
                    row[7]
                        .as_string()
                        .expect("Fail to parse previous block issuer"),
                ).expect("Fail to parse previous block issuer (2)"),
            ),
        })
    } else {
        None
    };
    let currency = row[3].as_string().expect("Fail to parse currency");
    let dividend = match row[12].as_integer() {
        Some(dividend) => Some(dividend as usize),
        None => None,
    };
    let json_identities: serde_json::Value = serde_json::from_str(
        row[20].as_string().expect("Fail to parse block identities"),
    ).expect("Fail to parse block identities (2)");
    let mut identities = Vec::new();
    for raw_idty in json_identities
        .as_array()
        .expect("Fail to parse block identities (3)")
    {
        identities
            .push(parse_compact_identity(&currency, &raw_idty).expect("Fail to parse block idty"));
    }
    let json_txs: serde_json::Value = serde_json::from_str(
        row[18].as_string().expect("Fail to parse block txs"),
    ).expect("Fail to parse block txs (2)");
    let mut transactions = Vec::new();
    for json_tx in json_txs.as_array().expect("Fail to parse block txs (3)") {
        transactions.push(parse_transaction(currency, &json_tx).expect("Fail to parse block tx"));
    }
    let previous_hash = match previous_header.clone() {
        Some(previous_header_) => previous_header_.hash.0,
        None => Hash::default(),
    };
    let previous_issuer = match previous_header {
        Some(previous_header_) => Some(previous_header_.issuer),
        None => None,
    };
    let excluded: serde_json::Value = serde_json::from_str(
        row[25].as_string().expect("Fail to parse excluded"),
    ).expect("Fail to parse excluded (2)");
    let uncompleted_block_doc = BlockDocument {
        nonce: row[17].as_integer().expect("Fail to parse nonce") as u64,
        number: current_header.number,
        pow_min: row[15].as_integer().expect("Fail to parse pow_min") as usize,
        time: row[14].as_integer().expect("Fail to parse time") as u64,
        median_time: row[11].as_integer().expect("Fail to parse median_time") as u64,
        members_count: row[9].as_integer().expect("Fail to parse members_count") as usize,
        monetary_mass: row[10]
            .as_string()
            .expect("Fail to parse monetary_mass")
            .parse()
            .expect("Fail to parse monetary_mass (2)"),
        unit_base: row[13].as_integer().expect("Fail to parse unit_base") as usize,
        issuers_count: row[28].as_integer().expect("Fail to parse issuers_count") as usize,
        issuers_frame: row[26].as_integer().expect("Fail to parse issuers_frame") as isize,
        issuers_frame_var: row[27]
            .as_integer()
            .expect("Fail to parse issuers_frame_var") as isize,
        currency: String::from(currency),
        issuers: vec![PubKey::Ed25519(
            ed25519::PublicKey::from_base58(row[4].as_string().expect("Fail to parse issuer"))
                .expect("Fail to parse issuer '2)"),
        )],
        signatures: vec![Sig::Ed25519(
            ed25519::Signature::from_base64(row[2].as_string().expect("Fail to parse signature"))
                .expect("Fail to parse signature (2)"),
        )],
        hash: Some(current_header.hash),
        parameters: None,
        previous_hash,
        previous_issuer,
        inner_hash: Some(
            Hash::from_hex(row[1].as_string().expect("Fail to parse block inner_hash"))
                .expect("Fail to parse block inner_hash (2)"),
        ),
        dividend,
        identities,
        joiners: duniter_dal::parsers::memberships::parse_memberships(
            currency,
            MembershipType::In(),
            row[21].as_string().expect("Fail to parse joiners"),
        ).expect("Fail to parse joiners (2)"),
        actives: duniter_dal::parsers::memberships::parse_memberships(
            currency,
            MembershipType::In(),
            row[22].as_string().expect("Fail to parse actives"),
        ).expect("Fail to parse actives (2)"),
        leavers: duniter_dal::parsers::memberships::parse_memberships(
            currency,
            MembershipType::In(),
            row[23].as_string().expect("Fail to parse leavers"),
        ).expect("Fail to parse leavers (2)"),
        revoked: Vec::new(),
        excluded: excluded
            .as_array()
            .expect("Fail to parse excluded (3)")
            .to_vec()
            .into_iter()
            .map(|e| {
                PubKey::Ed25519(
                    ed25519::PublicKey::from_base58(
                        e.as_str().expect("Fail to parse excluded (4)"),
                    ).expect("Fail to parse excluded (5)"),
                )
            })
            .collect(),
        certifications: Vec::new(),
        transactions,
        inner_hash_and_nonce_str: String::new(),
    };
    let revoked: serde_json::Value = serde_json::from_str(
        row[24].as_string().expect("Fail to parse revoked"),
    ).expect("Fail to parse revoked (2)");
    let certifications: serde_json::Value = serde_json::from_str(
        row[19].as_string().expect("Fail to parse certifications"),
    ).expect("Fail to parse certifications (2)");
    // return NetworkBlock
    NetworkBlock::V10(Box::new(NetworkBlockV10 {
        uncompleted_block_doc,
        revoked: revoked
            .as_array()
            .expect("Fail to parse revoked (3)")
            .to_vec(),
        certifications: certifications
            .as_array()
            .expect("Fail to parse certifications (3)")
            .to_vec(),
    }))
}

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

//! Databases explorer

use crate::*;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::BlockNumber;
use dup_crypto::keys::*;
use durs_bc_db_reader::BcDbRo;
use durs_wot::data::rusty::RustyWebOfTrust;
use durs_wot::data::WebOfTrust;
use durs_wot::operations::distance::{DistanceCalculator, WotDistance, WotDistanceParameters};
use prettytable::Table;
use std::str::FromStr;
use std::time::*;
use unwrap::unwrap;

/// Error message for empty blockchain case
pub static EMPTY_BLOCKCHAIN: &str = "No blockchain, please sync your node to get a blockchain.";

static PUB_KEY: &str = "PUBKEY";
static BLOCK: &str = "BLOCK";
static USERNAME: &str = "USERNAME";

#[derive(Debug, Copy, Clone)]
/// Query for blockchain databases explorer
pub enum DbExBcQuery {
    /// Count blocks per issuer
    CountBlocksPerIssuer,
}

#[derive(Debug, Clone)]
/// Query for tx databases explorer
pub enum DbExTxQuery {
    /// Ask balance of an address (pubkey or uid)
    Balance(String),
}

#[derive(Debug, Clone)]
/// Query for wot databases explorer
pub enum DbExWotQuery {
    /// Ask distance of all members
    AllDistances(bool),
    /// Show members expire date
    ExpireMembers(bool),
    /// Show members list
    ListMembers(bool),
    /// Ask member datas
    MemberDatas(UidOrPubkey),
}

/// Username or public key
#[derive(Debug, Clone)]
pub enum UidOrPubkey {
    /// Public key
    Pubkey(PubKey),
    /// Username
    Uid(String),
}

impl From<String> for UidOrPubkey {
    fn from(s: String) -> Self {
        if let Ok(pubkey) = PubKey::from_str(&s) {
            Self::Pubkey(pubkey)
        } else {
            Self::Uid(s)
        }
    }
}

#[derive(Debug, Clone)]
/// Query for databases explorer
pub enum DbExQuery {
    /// Blockchain query
    BcQuery(DbExBcQuery),
    /// Fork tree query
    ForkTreeQuery,
    /// Tx query
    TxQuery(DbExTxQuery),
    /// Wot query
    WotQuery(DbExWotQuery),
}

fn open_bc_db_ro(profile_path: PathBuf) -> Option<BcDbRo> {
    // Get db path
    let db_path = durs_conf::get_blockchain_db_path(profile_path);

    match durs_bc_db_reader::open_db_ro(&db_path) {
        Ok(db) => Some(db),
        Err(DbError::DBNotExist) => {
            println!("DB not exist, please sync.");
            None
        }
        Err(e) => {
            println!("Fail to open DB: {:?}", e);
            None
        }
    }
}

/// Execute DbExQuery
pub fn dbex(profile_path: PathBuf, csv: bool, query: &DbExQuery) {
    match *query {
        DbExQuery::ForkTreeQuery => dbex_fork_tree(profile_path, csv),
        DbExQuery::BcQuery(bc_query) => {
            dbex_bc(profile_path, csv, bc_query).expect("Error: fail to open DB.")
        }
        DbExQuery::TxQuery(ref tx_query) => dbex_tx(profile_path, csv, tx_query),
        DbExQuery::WotQuery(ref wot_query) => dbex_wot(profile_path, csv, wot_query),
    }
}

/// Execute DbExBcQuery
pub fn dbex_bc(profile_path: PathBuf, _csv: bool, _query: DbExBcQuery) -> Result<(), DbError> {
    // Get db path
    let db_path = durs_conf::get_blockchain_db_path(profile_path);

    // Open databases
    let load_dbs_begin = SystemTime::now();
    let db = durs_bc_db_reader::open_db_ro(&db_path.as_path())?;

    let load_dbs_duration = SystemTime::now()
        .duration_since(load_dbs_begin)
        .expect("duration_since error !");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_millis()
    );

    if let Some(current_blockstamp) =
        durs_bc_db_reader::current_meta_datas::get_current_blockstamp(&db)?
    {
        println!("Current block: #{}.", current_blockstamp);
        if let Some(current_block) =
            durs_bc_db_reader::blocks::get_block_in_local_blockchain(&db, current_blockstamp.id)?
        {
            let map_pubkey = durs_bc_db_reader::blocks::get_current_frame(&current_block, &db)?;

            let mut vec = map_pubkey.iter().collect::<Vec<(&PubKey, &usize)>>();
            vec.sort_by(|a, b| b.1.cmp(&a.1));

            if _csv {
                println!("{},{},{}", &BLOCK, &USERNAME, &PUB_KEY);
                for (pub_key, v) in &vec {
                    if let Ok(Some(identity)) =
                        durs_bc_db_reader::indexes::identities::get_identity_by_pubkey(
                            &db, &pub_key,
                        )
                    {
                        println!(
                            "{},{},{}",
                            v,
                            identity.idty_doc.username(),
                            pub_key.to_string()
                        );
                    }
                }
            } else {
                let mut table = Table::new();
                table.add_row(row![&BLOCK, &USERNAME, &PUB_KEY]);
                for (pub_key, v) in &vec {
                    if let Ok(Some(identity)) =
                        durs_bc_db_reader::indexes::identities::get_identity_by_pubkey(
                            &db, &pub_key,
                        )
                    {
                        table.add_row(row![v, identity.idty_doc.username(), pub_key.to_string()]);
                    }
                }
                table.printstd();
            }
        }
    }

    Ok(())
}

/// Print fork tree
pub fn dbex_fork_tree(profile_path: PathBuf, _csv: bool) {
    // Open DB
    let load_db_begin = SystemTime::now();
    let db = if let Some(db) = open_bc_db_ro(profile_path) {
        db
    } else {
        return;
    };
    let load_db_duration = SystemTime::now()
        .duration_since(load_db_begin)
        .expect("duration_since error !");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_db_duration.as_secs(),
        load_db_duration.subsec_millis()
    );
    let fork_tree =
        durs_bc_db_reader::current_meta_datas::get_fork_tree(&db).expect("fail to get fork tree");
    // Print all fork branches
    for (tree_node_id, blockstamp) in fork_tree.get_sheets() {
        debug!(
            "fork_tree.get_fork_branch({:?}, {})",
            tree_node_id, blockstamp
        );
        let branch = fork_tree.get_fork_branch(tree_node_id);
        if !branch.is_empty() {
            println!("Fork branch #{}:", blockstamp);
            println!("{:#?}", branch);
        }
    }
}

/// Execute DbExTxQuery
pub fn dbex_tx(profile_path: PathBuf, _csv: bool, _query: &DbExTxQuery) {
    // Get db path
    let _db_path = durs_conf::get_blockchain_db_path(profile_path.clone());

    unimplemented!();

    /*// Open DB
    let load_db_begin = SystemTime::now();
    let db = if let Some(db) = open_bc_db_ro(profile_path) {
        db
    } else {
        return;
    };
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_db_begin)
        .expect("duration_since error !");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_millis()
    );
    let req_process_begin = SystemTime::now();
    match *query {
        DbExTxQuery::Balance(ref address_str) => {
            let pubkey = if let Ok(ed25519_pubkey) = ed25519::PublicKey::from_base58(address_str) {
                PubKey::Ed25519(ed25519_pubkey)
            } else if let Some(pubkey) =
                durs_bc_db_reader::indexes::identities::get_wot_id_from_uid(&db, address_str)
                    .expect("get_uid : DbError")
            {
                pubkey
            } else {
                println!("This address doesn't exist!");
                return;
            };
            let address = UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(pubkey));
            let address_balance = durs_bc_db_reader::indexes::balance::get_address_balance(
                &currency_databases.balances_db,
                &address,
            )
            .expect("get_address_balance : DbError")
            .expect("Address not found in balances DB.");
            println!(
                "Balance={},{} Ğ1",
                (address_balance.0).0 / 100,
                (address_balance.0).0 % 100
            );
        }
    }

    let req_process_duration = SystemTime::now()
        .duration_since(req_process_begin)
        .expect("duration_since error");
    println!(
        "Request processed in  {}.{:06} seconds.",
        req_process_duration.as_secs(),
        req_process_duration.subsec_micros()
    );*/
}

/// Execute DbExWotQuery
pub fn dbex_wot(profile_path: PathBuf, csv: bool, query: &DbExWotQuery) {
    // Get db path
    let db_path = durs_conf::get_blockchain_db_path(profile_path.clone());

    // Open DB
    let load_db_begin = SystemTime::now();
    let db = if let Some(db) = open_bc_db_ro(profile_path.clone()) {
        db
    } else {
        return;
    };
    let wot_databases = WotsV10DBs::open(Some(&db_path));
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_db_begin)
        .expect("duration_since error");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_millis()
    );

    // Get currency parameters
    let currency_params_db_datas =
        dubp_currency_params::db::get_currency_params(durs_conf::get_datas_path(profile_path))
            .expect("Fail to parse currency params !");
    if currency_params_db_datas.is_none() {
        println!("{}", EMPTY_BLOCKCHAIN);
        return;
    }
    let currency_params = unwrap!(currency_params_db_datas).1;

    // get wot_index
    let wot_index = durs_bc_db_reader::indexes::identities::get_wot_index(&db).expect("DbError");

    // get wot_reverse_index
    let wot_reverse_index: HashMap<WotId, &PubKey> =
        wot_index.iter().map(|(p, id)| (*id, p)).collect();

    // get wot uid index
    let wot_uid_index =
        durs_bc_db_reader::indexes::identities::get_wot_uid_index(&db).expect("DbError");

    // Open wot db
    let wot_db = BinFreeStructDb::File(
        open_free_struct_file_db::<RustyWebOfTrust>(&db_path, "wot.db")
            .expect("Fail to open WotDB !"),
    );

    // Print wot blockstamp
    //println!("Wot : Current blockstamp = {}.", wot_blockstamp);

    // Get members count
    let members_count = wot_db
        .read(WebOfTrust::get_enabled)
        .expect("Fail to read WotDB")
        .len();

    match *query {
        DbExWotQuery::AllDistances(ref reverse) => {
            println!("compute distances...");
            let compute_distances_begin = SystemTime::now();
            let mut distances_datas: Vec<(WotId, WotDistance)> = wot_db
                .read(|db| {
                    db.get_enabled()
                        .iter()
                        .map(|wot_id| {
                            (
                                *wot_id,
                                DISTANCE_CALCULATOR
                                    .compute_distance(
                                        db,
                                        WotDistanceParameters {
                                            node: *wot_id,
                                            sentry_requirement: 5,
                                            step_max: currency_params.step_max as u32,
                                            x_percent: currency_params.x_percent,
                                        },
                                    )
                                    .expect("Fail to get distance !"),
                            )
                        })
                        .collect()
                })
                .expect("Fail to read WotDB");
            let compute_distances_duration = SystemTime::now()
                .duration_since(compute_distances_begin)
                .expect("duration_since error");
            if *reverse {
                distances_datas.sort_unstable_by(|(_, d1), (_, d2)| d1.success.cmp(&d2.success));
            } else {
                distances_datas.sort_unstable_by(|(_, d1), (_, d2)| d2.success.cmp(&d1.success));
            }
            for (wot_id, distance_datas) in distances_datas {
                let distance_percent: f64 =
                    f64::from(distance_datas.success) / f64::from(distance_datas.sentries) * 100.0;
                if csv {
                    println!("{}, {}", wot_uid_index[&wot_id], distance_percent,);
                } else {
                    println!(
                        "{} -> distance: {:.2}% ({}/{})",
                        wot_uid_index[&wot_id],
                        distance_percent,
                        distance_datas.success,
                        distance_datas.sentries
                    );
                }
            }
            println!(
                "compute_distances_duration = {},{:03}.",
                compute_distances_duration.as_secs(),
                compute_distances_duration.subsec_millis()
            );
        }
        DbExWotQuery::ExpireMembers(ref reverse) => {
            // Open blockchain database
            let db = durs_bc_db_reader::open_db_ro(&db_path.as_path()).expect("Fail to open DB.");
            // Get blocks_times
            let all_blocks = durs_bc_db_reader::blocks::get_blocks_in_local_blockchain(
                &db,
                BlockNumber(0),
                10_000_000,
            )
            .expect("Fail to get all blocks");
            let current_bc_time = all_blocks.last().expect("empty blockchain").common_time();
            let blocks_times: HashMap<BlockNumber, u64> = all_blocks
                .iter()
                .map(|block| (block.number(), block.common_time()))
                .collect();
            // Get expire_dates
            let min_created_ms_time = current_bc_time - currency_params.ms_validity;
            let mut expire_dates: Vec<(WotId, u64)> = wot_databases
                .ms_db
                .read(|db| {
                    let mut expire_dates = Vec::new();
                    for (block_id, nodes_ids) in db {
                        let created_ms_time = blocks_times[&block_id.0];
                        if created_ms_time > min_created_ms_time {
                            for node_id in nodes_ids {
                                expire_dates.push((
                                    *node_id,
                                    created_ms_time + currency_params.ms_validity,
                                ));
                            }
                        }
                    }
                    expire_dates
                })
                .expect("Fail to read ms db");
            if *reverse {
                expire_dates.sort_unstable_by(|(_, d1), (_, d2)| d1.cmp(&d2));
            } else {
                expire_dates.sort_unstable_by(|(_, d1), (_, d2)| d2.cmp(&d1));
            }
            for (node_id, expire_date) in expire_dates {
                println!("{}, {}", wot_uid_index[&node_id], expire_date);
            }
        }
        DbExWotQuery::MemberDatas(ref uid_or_pubkey) => {
            println!(" Members count = {}.", members_count);
            let wot_id_opt = match uid_or_pubkey {
                UidOrPubkey::Uid(ref uid) => {
                    durs_bc_db_reader::indexes::identities::get_wot_id_from_uid(&db, uid)
                        .expect("get_wot_id_from_uid() : DbError !")
                }
                UidOrPubkey::Pubkey(ref pubkey) => wot_index.get(pubkey).copied(),
            };
            if let Some(wot_id) = wot_id_opt {
                let idty =
                    durs_bc_db_reader::indexes::identities::get_identity_by_wot_id(&db, wot_id)
                        .expect("DB error: ")
                        .expect("DB corrupted: all WotId must be point to an identity.");

                println!(
                    "{} : wot_id={}, pubkey={}.",
                    idty.idty_doc.username(),
                    wot_id.0,
                    idty.idty_doc.issuers()[0].to_string()
                );
                let distance_datas = wot_db
                    .read(|db| {
                        DISTANCE_CALCULATOR.compute_distance(
                            db,
                            WotDistanceParameters {
                                node: wot_id,
                                sentry_requirement: 5,
                                step_max: currency_params.step_max as u32,
                                x_percent: currency_params.x_percent,
                            },
                        )
                    })
                    .expect("Fail to read WotDB.")
                    .expect("Fail to get distance.");
                let distance_percent: f64 =
                    f64::from(distance_datas.success) / f64::from(distance_datas.sentries) * 100.0;
                println!(
                    "Distance {:.2}% ({}/{})",
                    distance_percent, distance_datas.success, distance_datas.sentries
                );
                let sources = wot_db
                    .read(|db| db.get_links_source(wot_id))
                    .expect("Fail to read WotDB")
                    .expect("Fail to get links source !");
                println!("Certifiers : {}", sources.len());
                for (i, source) in sources.iter().enumerate() {
                    let source_uid = durs_bc_db_reader::indexes::identities::get_uid(
                        &db,
                        wot_reverse_index[&source],
                    )
                    .expect("get_uid() : DbError")
                    .expect("Not found source_uid !");
                    println!("{}: {}", i + 1, source_uid);
                }
            } else {
                println!("{:?} not found !", uid_or_pubkey);
            }
        }
        _ => {}
    }
}

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

use dubp_documents::v10::transaction::*;
use duniter_module::DuniterConf;
use dup_crypto::keys::*;
use durs_blockchain_dal::identity::DALIdentity;
use durs_wot::data::rusty::RustyWebOfTrust;
use durs_wot::operations::distance::{DistanceCalculator, WotDistance, WotDistanceParameters};
use std::time::*;
use *;

#[derive(Debug, Clone)]
/// Query for wot databases explorer
pub enum DBExWotQuery {
    /// Ask distance of all members
    AllDistances(bool),
    /// Show members expire date
    ExpireMembers(bool),
    /// Show members list
    ListMembers(bool),
    /// Ask member datas
    MemberDatas(String),
}

#[derive(Debug, Clone)]
/// Query for tx databases explorer
pub enum DBExTxQuery {
    /// Ask balance of an address (pubkey or uid)
    Balance(String),
}

#[derive(Debug, Clone)]
/// Query for databases explorer
pub enum DBExQuery {
    /// Wot query
    WotQuery(DBExWotQuery),
    /// Tx query
    TxQuery(DBExTxQuery),
}

pub fn dbex<DC: DuniterConf>(profile: &str, conf: &DC, csv: bool, query: &DBExQuery) {
    match *query {
        DBExQuery::WotQuery(ref wot_query) => dbex_wot(profile, conf, csv, wot_query),
        DBExQuery::TxQuery(ref tx_query) => dbex_tx(profile, conf, csv, tx_query),
    }
}

pub fn dbex_tx<DC: DuniterConf>(profile: &str, conf: &DC, _csv: bool, query: &DBExTxQuery) {
    // Get db path
    let db_path = duniter_conf::get_blockchain_db_path(profile, &conf.currency());

    // Open databases
    let load_dbs_begin = SystemTime::now();
    //let blocks_databases = BlocksV10DBs::open(Some(&db_path));
    let currency_databases = CurrencyV10DBs::open(Some(&db_path));
    let wot_databases = WotsV10DBs::open(Some(&db_path));
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_dbs_begin)
        .expect("duration_since error !");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_millis()
    );
    let req_process_begin = SystemTime::now();
    match *query {
        DBExTxQuery::Balance(ref address_str) => {
            let pubkey = if let Ok(ed25519_pubkey) = ed25519::PublicKey::from_base58(address_str) {
                PubKey::Ed25519(ed25519_pubkey)
            } else if let Some(pubkey) = durs_blockchain_dal::identity::get_pubkey_from_uid(
                &wot_databases.identities_db,
                address_str,
            )
            .expect("get_uid : DALError")
            {
                pubkey
            } else {
                println!("This address doesn't exist !");
                return;
            };
            let address = UTXOConditionsGroup::Single(TransactionOutputCondition::Sig(pubkey));
            let address_balance = durs_blockchain_dal::balance::get_address_balance(
                &currency_databases.balances_db,
                &address,
            )
            .expect("get_address_balance : DALError")
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
    );
}

pub fn dbex_wot<DC: DuniterConf>(profile: &str, conf: &DC, csv: bool, query: &DBExWotQuery) {
    // Get db path
    let db_path = duniter_conf::get_blockchain_db_path(profile, &conf.currency());

    // Open databases
    let load_dbs_begin = SystemTime::now();
    let currency_params_db =
        open_db::<CurrencyParamsV10Datas>(&db_path, "params.db").expect("Fail to open params db");
    let wot_databases = WotsV10DBs::open(Some(&db_path));
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_dbs_begin)
        .expect("duration_since error");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_millis()
    );

    // Get currency parameters
    let currency_params = currency_params_db
        .read(|db| CurrencyParameters::from(db.clone()))
        .expect("Fail to parse currency params !");

    // get wot_index
    let wot_index = DALIdentity::get_wot_index(&wot_databases.identities_db).expect("DALError");

    // get wot_reverse_index
    let wot_reverse_index: HashMap<NodeId, &PubKey> =
        wot_index.iter().map(|(p, id)| (*id, p)).collect();

    // get wot uid index
    let wot_uid_index: HashMap<NodeId, String> = wot_databases
        .identities_db
        .read(|db| {
            db.iter()
                .map(|(_, idty)| (idty.wot_id, String::from(idty.idty_doc.username())))
                .collect()
        })
        .expect("Fail to read IdentitiesDB !");

    // Open wot db
    let wot_db = open_wot_db::<RustyWebOfTrust>(Some(&db_path)).expect("Fail to open WotDB !");

    // Print wot blockstamp
    //println!("Wot : Current blockstamp = {}.", wot_blockstamp);

    // Get members count
    let members_count = wot_db
        .read(|db| db.get_enabled())
        .expect("Fail to read WotDB")
        .len();

    match *query {
        DBExWotQuery::AllDistances(ref reverse) => {
            println!("compute distances...");
            let compute_distances_begin = SystemTime::now();
            let mut distances_datas: Vec<(NodeId, WotDistance)> = wot_db
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
        DBExWotQuery::ExpireMembers(ref reverse) => {
            // Open blockchain database
            let blockchain_db = open_db::<LocalBlockchainV10Datas>(&db_path, "blockchain.db")
                .expect("Fail to open blockchain db");
            // Get blocks_times
            let (current_bc_time, blocks_times): (u64, HashMap<BlockId, u64>) = blockchain_db
                .read(|db| {
                    (
                        db[&BlockId(db.len() as u32 - 1)].block.median_time,
                        db.iter()
                            .map(|(block_id, dal_block)| (*block_id, dal_block.block.median_time))
                            .collect(),
                    )
                })
                .expect("Fail to read blockchain db");
            // Get expire_dates
            let min_created_ms_time = current_bc_time - currency_params.ms_validity;
            let mut expire_dates: Vec<(NodeId, u64)> = wot_databases
                .ms_db
                .read(|db| {
                    let mut expire_dates = Vec::new();
                    for (block_id, nodes_ids) in db {
                        let created_ms_time = blocks_times[&block_id];
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
        DBExWotQuery::MemberDatas(ref uid) => {
            println!(" Members count = {}.", members_count);
            if let Some(pubkey) = durs_blockchain_dal::identity::get_pubkey_from_uid(
                &wot_databases.identities_db,
                uid,
            )
            .expect("get_pubkey_from_uid() : DALError !")
            {
                let wot_id = wot_index[&pubkey];
                println!(
                    "{} : wot_id={}, pubkey={}.",
                    uid,
                    wot_id.0,
                    pubkey.to_string()
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
                    .expect("Fail to read WotDB")
                    .expect("Fail to get distance !");
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
                    let source_uid = durs_blockchain_dal::identity::get_uid(
                        &wot_databases.identities_db,
                        *(wot_reverse_index[&source]),
                    )
                    .expect("get_uid() : DALError")
                    .expect("Not found source_uid !");
                    println!("{}: {}", i + 1, source_uid);
                }
            } else {
                println!("Uid \"{}\" not found !", uid);
            }
        }
        _ => {}
    }
}

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

use duniter_crypto::keys::*;
use duniter_dal::identity::DALIdentity;
use duniter_documents::blockchain::v10::documents::transaction::*;
use duniter_documents::Blockstamp;
use duniter_module::DuniterConf;
use duniter_wotb::data::rusty::RustyWebOfTrust;
use std::time::*;
use *;

#[derive(Debug, Clone)]
/// Query for wot databases explorer
pub enum DBExWotQuery {
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

pub fn dbex(conf: &DuniterConf, query: &DBExQuery) {
    match *query {
        DBExQuery::WotQuery(ref wot_query) => dbex_wot(conf, wot_query),
        DBExQuery::TxQuery(ref tx_query) => dbex_tx(conf, tx_query),
    }
}

pub fn dbex_tx(conf: &DuniterConf, query: &DBExTxQuery) {
    // Get db path
    let db_path = duniter_conf::get_blockchain_db_path(conf.profile().as_str(), &conf.currency());

    // Open databases
    let load_dbs_begin = SystemTime::now();
    //let blocks_databases = BlocksV10DBs::open(&db_path, false);
    let currency_databases = CurrencyV10DBs::open(&db_path, false);
    let wot_databases = WotsV10DBs::open(&db_path, false);
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_dbs_begin)
        .expect("duration_since error !");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_nanos() / 1_000_000
    );
    let req_process_begin = SystemTime::now();

    match *query {
        DBExTxQuery::Balance(ref address_str) => {
            let pubkey = if let Ok(ed25519_pubkey) = ed25519::PublicKey::from_base58(address_str) {
                PubKey::Ed25519(ed25519_pubkey)
            } else if let Some(pubkey) = duniter_dal::identity::get_pubkey_from_uid(
                &wot_databases.identities_db,
                address_str,
            ).expect("get_uid : DALError")
            {
                pubkey
            } else {
                println!("This address doesn't exist !");
                return;
            };
            let address =
                TransactionOutputConditionGroup::Single(TransactionOutputCondition::Sig(pubkey));
            let address_balance = duniter_dal::balance::get_address_balance(
                &currency_databases.balances_db,
                &address,
            ).expect("get_address_balance : DALError")
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
        req_process_duration.subsec_nanos() / 1_000
    );
}

pub fn dbex_wot(conf: &DuniterConf, query: &DBExWotQuery) {
    // Get db path
    let db_path = duniter_conf::get_blockchain_db_path(conf.profile().as_str(), &conf.currency());

    // Open databases
    let load_dbs_begin = SystemTime::now();
    //let blocks_databases = BlocksV10DBs::open(&db_path, false);
    let wot_databases = WotsV10DBs::open(&db_path, false);
    let load_dbs_duration = SystemTime::now()
        .duration_since(load_dbs_begin)
        .expect("duration_since error");
    println!(
        "Databases loaded in {}.{:03} seconds.",
        load_dbs_duration.as_secs(),
        load_dbs_duration.subsec_nanos() / 1_000_000
    );
    let req_process_begin = SystemTime::now();

    // get wot_index
    let wot_index = DALIdentity::get_wotb_index(&wot_databases.identities_db).expect("DALError");

    // get wot_reverse_index
    let wot_reverse_index: HashMap<NodeId, &PubKey> =
        wot_index.iter().map(|(p, id)| (*id, p)).collect();

    // Get wot path
    let wot_path = duniter_conf::get_wot_path(conf.profile().clone().to_string(), &conf.currency());

    // Open wot file
    let (wot, wot_blockstamp): (RustyWebOfTrust, Blockstamp) =
        open_wot_file(&WOT_FILE_FORMATER, &wot_path, *INFINITE_SIG_STOCK);

    // Print wot blockstamp
    println!("Wot : Current blockstamp = {}.", wot_blockstamp);

    // Print members count
    let members_count = wot.get_enabled().len();
    println!(" Members count = {}.", members_count);

    match *query {
        DBExWotQuery::MemberDatas(ref uid) => {
            if let Some(pubkey) =
                duniter_dal::identity::get_pubkey_from_uid(&wot_databases.identities_db, uid)
                    .expect("get_pubkey_from_uid() : DALError !")
            {
                let wot_id = wot_index[&pubkey];
                println!(
                    "{} : wot_id={}, pubkey={}.",
                    uid,
                    wot_id.0,
                    pubkey.to_string()
                );
                let sources = wot
                    .get_links_source(wot_id)
                    .expect("Fail to get links source !");
                println!("Certifiers : {}", sources.len());
                for (i, source) in sources.iter().enumerate() {
                    let source_uid = duniter_dal::identity::get_uid(
                        &wot_databases.identities_db,
                        *(wot_reverse_index[&source]),
                    ).expect("get_uid() : DALError")
                        .expect("Not found source_uid !");
                    println!("{}: {}", i + 1, source_uid);
                }
            } else {
                println!("Uid \"{}\" not found !", uid);
            }
        }
    }
    let req_process_duration = SystemTime::now()
        .duration_since(req_process_begin)
        .expect("duration_since error");
    println!(
        "Request processed in  {}.{:06} seconds.",
        req_process_duration.as_secs(),
        req_process_duration.subsec_nanos() / 1_000
    );
}

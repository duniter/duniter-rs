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

//! Defined the few global types used by all modules,
//! as well as the DuniterModule trait that all modules must implement.

#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(feature = "cargo-clippy", allow(implicit_hasher))]
#![cfg_attr(feature = "exp", allow(warnings))]
#![deny(
    missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_wotb;
extern crate serde;
extern crate sqlite;

pub mod block;
pub mod constants;
pub mod dal_event;
pub mod dal_requests;
pub mod identity;
pub mod parsers;
pub mod tools;
pub mod writers;

use duniter_crypto::keys::{PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::{BlockHash, BlockId, Blockstamp, Hash};
use duniter_wotb::operations::file::FileFormater;
use duniter_wotb::{NodeId, WebOfTrust};
use std::fmt::Debug;
use std::marker;
use std::path::PathBuf;

use self::block::DALBlock;

pub struct DuniterDB(pub sqlite::Connection);

impl Debug for DuniterDB {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "DuniterDB {{ }}")
    }
}

pub trait FromJsonValue
where
    Self: marker::Sized,
{
    fn from_json_value(value: &serde_json::Value) -> Option<Self>;
}

pub trait WriteToDuniterDB {
    fn write(&self, db: &DuniterDB, written_blockstamp: Blockstamp, written_timestamp: u64);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ForkState {
    Free(),
    Full(),
    Isolate(),
}

#[derive(Debug, Clone)]
pub struct WotState {
    pub block_number: u32,
    pub block_hash: String,
    pub sentries_count: usize,
    pub average_density: usize,
    pub average_distance: usize,
    pub distances: Vec<usize>,
    pub average_connectivity: usize,
    pub connectivities: Vec<usize>,
    pub average_centrality: usize,
    pub centralities: Vec<u64>,
}

fn _use_json_macro() -> serde_json::Value {
    json!({})
}

pub fn open_db(db_path: &PathBuf, memory_mode: bool) -> Result<DuniterDB, sqlite::Error> {
    let conn: sqlite::Connection;
    if memory_mode || !db_path.as_path().exists() {
        if memory_mode {
            conn = sqlite::open(":memory:")?;
        } else {
            conn = sqlite::open(db_path.as_path())?;
        }
        //conn.execute("PRAGMA synchronous = 0;")
        //.expect("Fail to configure SQLite DB (PRAGMA) !");
        conn.execute(
            "
        CREATE TABLE wot_history (block_number INTEGER, block_hash TEXT, sentries_count INTEGER,
            average_density INTEGER, average_distance INTEGER,
            distances TEXT, average_connectivity INTEGER, connectivities TEXT,
            average_centrality INTEGER, centralities TEXT);
        CREATE TABLE blocks (fork INTEGER, isolate INTEGER, version INTEGER, nonce INTEGER, number INTEGER,
            pow_min INTEGER, time INTEGER, median_time INTEGER, members_count INTEGER,
            monetary_mass INTEGER, unit_base INTEGER, issuers_count INTEGER, issuers_frame INTEGER,
            issuers_frame_var INTEGER, median_frame INTEGER, second_tiercile_frame INTEGER,
            currency TEXT, issuer TEXT, signature TEXT, hash TEXT, previous_hash TEXT, inner_hash TEXT, dividend INTEGER, identities TEXT, joiners TEXT,
            actives TEXT, leavers TEXT, revoked TEXT, excluded TEXT, certifications TEXT,
            transactions TEXT);
        CREATE TABLE identities (wotb_id INTEGER, uid TEXT, pubkey TEXT, hash TEXT, sig TEXT,
            state INTEGER, created_on TEXT, joined_on TEXT, penultimate_renewed_on TEXT, last_renewed_on TEXT,
            expires_on INTEGER, revokes_on INTEGER, expired_on TEXT, revoked_on TEXT);
        CREATE TABLE certifications (pubkey_from TEXT, pubkey_to TEXT, created_on TEXT,
            signature TEXT, written_on TEXT, expires_on INTEGER, chainable_on INTEGER);
        ",
        )?;
    } else {
        conn = sqlite::open(db_path.as_path())?;
    }
    Ok(DuniterDB(conn))
}

pub fn close_db(db: &DuniterDB) {
    db.0
        .execute("PRAGMA optimize;")
        .expect("Fail to optimize SQLite DB !");
}

pub fn get_uid(db: &DuniterDB, wotb_id: NodeId) -> Option<String> {
    let mut cursor: sqlite::Cursor = db
        .0
        .prepare("SELECT uid FROM identities WHERE wotb_id=? AND state=0 LIMIT 1;")
        .expect("Request SQL get_current_block is wrong !")
        .cursor();
    cursor
        .bind(&[sqlite::Value::Integer(wotb_id.0 as i64)])
        .expect("0");
    if let Some(row) = cursor.next().expect("fait to get_uid() : cursor error") {
        Some(String::from(
            row[0]
                .as_string()
                .expect("get_uid: Fail to parse uid field in str !"),
        ))
    } else {
        None
    }
}

pub fn new_get_current_block(db: &DuniterDB) -> Option<BlockDocument> {
    let mut cursor: sqlite::Cursor = db.0
        .prepare(
            "SELECT version, nonce, number, pow_min, time, median_time, members_count, monetary_mass, unit_base, issuers_count, issuers_frame, issuers_frame_var, median_frame, second_tiercile_frame, currency, issuer, signature, hash, dividend, joiners, actives, leavers, revoked, excluded, certifications, transactions FROM blocks
            WHERE fork=0 ORDER BY median_time DESC LIMIT ?;",
        )
        .expect("Request SQL get_current_block is wrong !")
        .cursor();

    cursor.bind(&[sqlite::Value::Integer(1)]).expect("0");
    if let Some(row) = cursor.next().expect("1") {
        let dividend = row[18].as_integer().expect("dividend");
        let dividend = if dividend > 0 {
            Some(dividend as usize)
        } else {
            None
        };
        return Some(BlockDocument {
            nonce: row[1].as_integer().expect("nonce") as u64,
            number: BlockId(row[2].as_integer().expect("2") as u32),
            pow_min: row[3].as_integer().expect("version") as usize,
            time: row[4].as_integer().expect("time") as u64,
            median_time: row[5].as_integer().expect("median_time") as u64,
            members_count: row[6].as_integer().expect("7") as usize,
            monetary_mass: row[7].as_integer().expect("8") as usize,
            unit_base: row[8].as_integer().expect("unit_base") as usize,
            issuers_count: row[9].as_integer().expect("issuers_count") as usize,
            issuers_frame: row[10].as_integer().expect("issuers_frame") as isize,
            issuers_frame_var: row[11].as_integer().expect("issuers_frame_var") as isize,
            currency: row[14].as_string().expect("currency").to_string(),
            issuers: vec![PublicKey::from_base58(row[15].as_string().expect("issuer")).unwrap()],
            signatures: vec![
                Signature::from_base64(row[16].as_string().expect("signature")).unwrap(),
            ],
            hash: Some(BlockHash(
                Hash::from_hex(row[17].as_string().expect("hash")).unwrap(),
            )),
            parameters: None,
            previous_hash: Hash::default(),
            previous_issuer: None,
            inner_hash: None,
            dividend,
            identities: Vec::with_capacity(0),
            joiners: Vec::with_capacity(0),
            actives: Vec::with_capacity(0),
            leavers: Vec::with_capacity(0),
            revoked: Vec::with_capacity(0),
            excluded: Vec::with_capacity(0),
            certifications: Vec::with_capacity(0),
            transactions: Vec::with_capacity(0),
            inner_hash_and_nonce_str: String::new(),
        });
    }
    None
}

pub fn get_current_block(currency: &str, db: &DuniterDB) -> Option<DALBlock> {
    let mut cursor: sqlite::Cursor = db
        .0
        .prepare("SELECT number, hash FROM blocks WHERE fork=0 ORDER BY median_time DESC LIMIT ?;")
        .expect("Request SQL get_current_block is wrong !")
        .cursor();

    cursor.bind(&[sqlite::Value::Integer(1)]).expect("0");

    if let Some(row) = cursor.next().unwrap() {
        let blockstamp = Blockstamp {
            id: BlockId(row[0].as_integer().unwrap() as u32),
            hash: BlockHash(Hash::from_hex(row[1].as_string().unwrap()).unwrap()),
        };
        DALBlock::get_block(currency, db, &blockstamp)
    } else {
        None
    }
}

pub fn open_wot_file<W: WebOfTrust, WF: FileFormater>(
    file_formater: &WF,
    wot_path: &PathBuf,
) -> (W, Blockstamp) {
    if wot_path.as_path().exists() {
        match file_formater.from_file(
            wot_path.as_path().to_str().unwrap(),
            constants::G1_PARAMS.sig_stock as usize,
        ) {
            Ok((wot, binary_blockstamp)) => match ::std::str::from_utf8(&binary_blockstamp) {
                Ok(str_blockstamp) => (wot, Blockstamp::from_string(str_blockstamp).unwrap()),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            },
            Err(e) => panic!("Fatal Error : fail to read wot file : {:?}", e),
        }
    } else {
        (
            W::new(constants::G1_PARAMS.sig_stock as usize),
            Blockstamp::default(),
        )
    }
}

pub fn register_wot_state(db: &DuniterDB, wot_state: &WotState) {
    if wot_state.block_number != 1 {
        db.0
            .execute(format!(
                "INSERT INTO wot_history (block_number, block_hash, sentries_count,
                average_density, average_distance, distances,
                average_connectivity, connectivities, average_centrality, centralities)
                VALUES ({}, '{}', {}, {}, {}, '{}', {}, '{}', {}, '{}');",
                wot_state.block_number,
                wot_state.block_hash,
                wot_state.sentries_count,
                wot_state.average_density,
                wot_state.average_distance,
                serde_json::to_string(&wot_state.distances).unwrap(),
                wot_state.average_connectivity,
                serde_json::to_string(&wot_state.connectivities).unwrap(),
                wot_state.average_centrality,
                serde_json::to_string(&wot_state.centralities).unwrap(),
            ))
            .unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BlockchainError {
    UnexpectedBlockNumber(),
    UnknowError(),
}

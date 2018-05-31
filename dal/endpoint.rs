extern crate crypto;
extern crate duniter_crypto;
extern crate sqlite;

use std::time::Duration;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use self::duniter_crypto::keys::*;
use super::DuniterDB;
use super::WriteToDuniterDB;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DALEndpointApi {
    WS2P,
    //WS2PS,
    //WS2PTOR,
    //DASA,
    //BMA,
    //BMAS,
}

impl From<u32> for DALEndpointApi {
    fn from(integer: u32) -> Self {
        match integer {
            _ => DALEndpointApi::WS2P,
        }
    }
}

pub fn string_to_api(api: &str) -> Option<DALEndpointApi> {
    match api {
        "WS2P" => Some(DALEndpointApi::WS2P),
        //"WS2PS" => Some(DALEndpointApi::WS2PS),
        //"WS2PTOR" => Some(DALEndpointApi::WS2PTOR),
        //"DASA" => Some(DALEndpointApi::DASA),
        //"BASIC_MERKLED_API" => Some(DALEndpointApi::BMA),
        //"BMAS" => Some(DALEndpointApi::BMAS),
        &_ => None,
    }
}

pub fn api_to_integer(api: &DALEndpointApi) -> i64 {
    match *api {
        DALEndpointApi::WS2P => 0,
        //DALEndpointApi::WS2PS => 1,
        //DALEndpointApi::WS2PTOR => 2,
        //DALEndpointApi::DASA => 3,
        //DALEndpointApi::BMA => 4,
        //DALEndpointApi::BMAS => 5,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DALEndpoint {
    pub hash_full_id: String,
    pub status: u32,
    pub node_id: u32,
    pub pubkey: PubKey,
    pub api: DALEndpointApi,
    pub version: usize,
    pub endpoint: String,
    pub last_check: u64,
}

impl DALEndpoint {
    pub fn new(
        status: u32,
        node_id: u32,
        pubkey: PubKey,
        api: DALEndpointApi,
        version: usize,
        endpoint: String,
        last_check: Duration,
    ) -> DALEndpoint {
        let mut sha = Sha256::new();
        sha.input_str(&format!(
            "{}{}{}{}",
            node_id,
            pubkey,
            api_to_integer(&api),
            version
        ));
        DALEndpoint {
            hash_full_id: sha.result_str(),
            status,
            node_id,
            pubkey,
            api,
            version,
            endpoint,
            last_check: last_check.as_secs(),
        }
    }
    pub fn get_endpoints_for_api(db: &DuniterDB, api: DALEndpointApi) -> Vec<DALEndpoint> {
        let mut cursor:sqlite::Cursor = db.0
        .prepare("SELECT hash_full_id, status, node_id, pubkey, api, version, endpoint, last_check FROM endpoints WHERE api=? ORDER BY status DESC;")
        .expect("get_endpoints_for_api() : Error in SQL request !")
        .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(api_to_integer(&api))])
            .expect("get_endpoints_for_api() : Error in cursor binding !");
        let mut endpoints = Vec::new();
        while let Some(row) = cursor
            .next()
            .expect("get_endpoints_for_api() : Error in cursor.next()")
        {
            endpoints.push(DALEndpoint {
                hash_full_id: row[0].as_string().unwrap().to_string(),
                status: row[1].as_integer().unwrap() as u32,
                node_id: row[2].as_integer().unwrap() as u32,
                pubkey: PubKey::Ed25519(ed25519::PublicKey::from_base58(row[3].as_string().unwrap()).unwrap()),
                api: DALEndpointApi::from(row[4].as_integer().unwrap() as u32),
                version: row[5].as_integer().unwrap() as usize,
                endpoint: row[6].as_string().unwrap().to_string(),
                last_check: row[7].as_integer().unwrap() as u64,
            });
        }
        endpoints
    }
}

impl WriteToDuniterDB for DALEndpoint {
    fn write(
        &self,
        db: &DuniterDB,
        _written_blockstamp: super::block_v10::BlockStampV10,
        _written_timestamp: u64,
    ) {
        // Check if endpoint it's already written
        let mut cursor: sqlite::Cursor = db.0
            .prepare("SELECT status FROM endpoints WHERE hash_full_id=? ORDER BY status DESC;")
            .expect("get_endpoints_for_api() : Error in SQL request !")
            .cursor();
        cursor
            .bind(&[sqlite::Value::String(self.hash_full_id.clone())])
            .expect("get_endpoints_for_api() : Error in cursor binding !");

        // If endpoint it's already written, update status
        if let Some(row) = cursor
            .next()
            .expect("get_endpoints_for_api() : Error in cursor.next()")
        {
            if row[0].as_integer().unwrap() as u32 != self.status {
                db.0
                    .execute(format!(
                        "UPDATE endpoints SET status={} WHERE hash_full_id='{}'",
                        self.status, self.hash_full_id
                    ))
                    .unwrap();
            }
        } else {
            db.0
            .execute(
                format!(
                    "INSERT INTO endpoints (hash_full_id, status, node_id, pubkey, api, version, endpoint, last_check) VALUES ('{}', {}, {}, '{}', {}, {}, '{}', {});",
                    self.hash_full_id, self.status, self.node_id, self.pubkey.to_string(),
                    api_to_integer(&self.api), self.version, self.endpoint, self.last_check
                )
            )
            .unwrap();
        }
    }
}

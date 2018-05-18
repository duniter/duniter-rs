extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_wotb;
extern crate serde;
extern crate serde_json;
extern crate sqlite;

use self::duniter_documents::blockchain::v10::documents::BlockDocument;
use self::duniter_documents::blockchain::Document;
use super::super::block::DALBlock;
use super::super::DuniterDB;

pub fn write_network_block(
    db: &DuniterDB,
    block: &BlockDocument,
    fork: usize,
    isolate: bool,
    revoked: &Vec<serde_json::Value>,
    certifications: &Vec<serde_json::Value>,
) {
    db.0
        .execute(
            format!("INSERT INTO blocks (fork, isolate, version, nonce, number, pow_min, time, median_time, members_count, monetary_mass, unit_base, issuers_count, issuers_frame, issuers_frame_var, currency, issuer, signature, hash, previous_hash, inner_hash, dividend, identities, joiners, actives, leavers, revoked, excluded, certifications, transactions) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
                fork, if isolate { 1 } else { 0 }, 10,
                block.nonce, block.number, block.pow_min, block.time, block.median_time,
                block.members_count, block.monetary_mass, block.unit_base, block.issuers_count,
                block.issuers_frame, block.issuers_frame_var, block.currency, block.issuers[0],
                block.signatures[0].to_string(), block.hash.unwrap().0.to_string(),
                block.previous_hash.to_string(), block.inner_hash.unwrap().to_string(),
                block.dividend.unwrap_or(0),
                serde_json::to_string(&block.identities).unwrap(),
                serde_json::to_string(&block.joiners).unwrap(), serde_json::to_string(&block.actives).unwrap(),
                serde_json::to_string(&block.leavers).unwrap(), serde_json::to_string(revoked).unwrap(),
                serde_json::to_string(&block.excluded).unwrap(), serde_json::to_string(certifications).unwrap(),
                serde_json::to_string(&block.transactions).unwrap()
            ))
        .unwrap();
}

pub fn write(db: &DuniterDB, block: &BlockDocument, fork: usize, isolate: bool) {
    let mut insert = true;
    if fork == 0 {
        if let Some(_fork) = DALBlock::get_block_fork(db, &block.blockstamp()) {
            insert = false;
            db.0
                .execute(format!(
                    "UPDATE blocks SET fork=0 WHERE number={} AND hash='{}';",
                    block.number,
                    block.hash.unwrap().0.to_string()
                ))
                .unwrap();
        }
    }

    if insert {
        db.0
            .execute(
                format!("INSERT INTO blocks (fork, isolate, version, nonce, number, pow_min, time, median_time, members_count, monetary_mass, unit_base, issuers_count, issuers_frame, issuers_frame_var, currency, issuer, signature, hash, previous_hash, inner_hash, dividend, identities, joiners, actives, leavers, revoked, excluded, certifications, transactions) VALUES ({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}');",
                    fork, if isolate { 1 } else { 0 }, 10,
                    block.nonce, block.number, block.pow_min, block.time, block.median_time,
                    block.members_count, block.monetary_mass, block.unit_base, block.issuers_count,
                    block.issuers_frame, block.issuers_frame_var, block.currency, block.issuers[0],
                    block.signatures[0].to_string(), block.hash.unwrap().0.to_string(),
                    block.previous_hash.to_string(), block.inner_hash.unwrap().to_string(),
                    block.dividend.unwrap_or(0), serde_json::to_string(&block.identities).unwrap(),
                    serde_json::to_string(&block.joiners).unwrap(), serde_json::to_string(&block.actives).unwrap(),
                    serde_json::to_string(&block.leavers).unwrap(), serde_json::to_string(&block.revoked).unwrap(),
                    serde_json::to_string(&block.excluded).unwrap(), serde_json::to_string(&block.certifications).unwrap(),
                    serde_json::to_string(&block.transactions).unwrap()
                ),
            )
            .unwrap();
    }
}

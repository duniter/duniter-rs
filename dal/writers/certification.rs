extern crate serde;
extern crate serde_json;
extern crate sqlite;

use super::super::DuniterDB;
use duniter_crypto::keys::ed25519;
use duniter_documents::blockchain::v10::documents::certification::CompactCertificationDocument;
use duniter_documents::Blockstamp;

pub fn write_certification(
    cert: &CompactCertificationDocument,
    db: &DuniterDB,
    written_blockstamp: Blockstamp,
    written_timestamp: u64,
) {
    let mut cursor = db
        .0
        .prepare("SELECT median_time FROM blocks WHERE number=? AND fork=0 LIMIT 1;")
        .expect("invalid write_certification sql request")
        .cursor();

    cursor
        .bind(&[sqlite::Value::Integer(i64::from(cert.block_number.0))])
        .expect("convert blockstamp to timestamp failure at step 1 !");

    let mut created_timestamp: i64 = 0;
    if let Some(row) = cursor
        .next()
        .expect("convert blockstamp to timestamp failure at step 2 !")
    {
        created_timestamp = row[0]
            .as_integer()
            .expect("Fail to write cert, impossible to get created_timestamp !");
    }

    db.0
        .execute(
            format!("INSERT INTO certifications (pubkey_from, pubkey_to, created_on, signature, written_on, expires_on, chainable_on) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {});",
                cert.issuer, cert.target, cert.block_number.0, cert.signature,
                written_blockstamp.to_string(),
                created_timestamp+super::super::constants::G1_PARAMS.sig_validity,
                written_timestamp+super::super::constants::G1_PARAMS.sig_period
            ))
        .expect("Fail to execute INSERT certification !");
}

pub fn remove_certification(from: ed25519::PublicKey, to: ed25519::PublicKey, db: &DuniterDB) {
    db.0
        .execute(format!(
            "DELETE FROM certifications WHERE pubkey_from={} AND pubkey_to={}",
            from, to
        ))
        .expect("Fail to execute DELETE certification !");
}

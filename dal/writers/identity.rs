extern crate duniter_wotb;
extern crate sqlite;

use super::super::identity::DALIdentity;
use super::super::DuniterDB;
use duniter_documents::blockchain::Document;
use duniter_documents::Blockstamp;
use duniter_wotb::NodeId;

pub fn write(
    idty: &DALIdentity,
    wotb_id: &NodeId,
    db: &DuniterDB,
    _written_blockstamp: Blockstamp,
    _written_timestamp: u64,
) {
    let expired_on = match idty.expired_on {
        Some(ref tmp) => tmp.to_string(),
        None => String::from(""),
    };
    let revoked_on = match idty.revoked_on {
        Some(ref tmp) => tmp.to_string(),
        None => String::from(""),
    };
    db.0
        .execute(
            format!("INSERT INTO identities (wotb_id, uid, pubkey, hash, sig, state, created_on, joined_on, penultimate_renewed_on, last_renewed_on, expires_on, revokes_on, expired_on, revoked_on) VALUES ({}, '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', {}, {}, '{}', '{}');",
                (*wotb_id).0, idty.idty_doc.username(), idty.idty_doc.issuers()[0], idty.hash,
                idty.idty_doc.signatures()[0], idty.state,
                idty.idty_doc.blockstamp().to_string(),
                idty.joined_on.to_string(),
                idty.penultimate_renewed_on.to_string(),
                idty.last_renewed_on.to_string(),
                idty.expires_on, idty.revokes_on, expired_on, revoked_on
            ))
        .unwrap();
}

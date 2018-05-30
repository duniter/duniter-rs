extern crate sqlite;

use super::block::{blockstamp_to_timestamp, DALBlock};
use super::DuniterDB;
use duniter_crypto::keys::{ed25519, PublicKey, Signature};
use duniter_documents::blockchain::v10::documents::identity::IdentityDocumentBuilder;
use duniter_documents::blockchain::v10::documents::IdentityDocument;
use duniter_documents::blockchain::{Document, DocumentBuilder};
use duniter_documents::Blockstamp;
use duniter_wotb::NodeId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DALIdentity {
    pub hash: String,
    pub state: isize,
    pub joined_on: Blockstamp,
    pub penultimate_renewed_on: Blockstamp,
    pub last_renewed_on: Blockstamp,
    pub expires_on: u64,
    pub revokes_on: u64,
    pub expired_on: Option<Blockstamp>,
    pub revoked_on: Option<Blockstamp>,
    pub idty_doc: IdentityDocument,
}

impl DALIdentity {
    pub fn exclude_identity(
        db: &DuniterDB,
        wotb_id: NodeId,
        renewal_blockstamp: Blockstamp,
        revert: bool,
    ) {
        let state = if revert { 0 } else { 1 };
        let expired_on = if revert {
            None
        } else {
            Some(renewal_blockstamp)
        };
        let mut cursor = db
            .0
            .prepare("UPDATE identities SET state=?, expired_on=?  WHERE wotb_id=?;")
            .expect("Fail to exclude idty !")
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(i64::from(state)),
                sqlite::Value::String(expired_on.unwrap_or_else(Blockstamp::default).to_string()),
                sqlite::Value::Integer(wotb_id.0 as i64),
            ])
            .expect("Fail to exclude idty !");
    }

    pub fn get_wotb_index(db: &DuniterDB) -> HashMap<ed25519::PublicKey, NodeId> {
        let mut wotb_index: HashMap<ed25519::PublicKey, NodeId> = HashMap::new();

        let mut cursor = db
            .0
            .prepare("SELECT wotb_id, pubkey FROM identities ORDER BY wotb_id ASC;")
            .unwrap()
            .cursor();

        while let Some(row) = cursor.next().unwrap() {
            wotb_index.insert(
                PublicKey::from_base58(row[1].as_string().unwrap()).unwrap(),
                NodeId(row[0].as_integer().unwrap() as usize),
            );
        }
        wotb_index
    }

    pub fn create_identity(
        db: &DuniterDB,
        idty_doc: &IdentityDocument,
        current_blockstamp: Blockstamp,
    ) -> DALIdentity {
        let created_on = idty_doc.blockstamp();
        let created_time = blockstamp_to_timestamp(&created_on, &db)
            .expect("convert blockstamp to timestamp failure !");

        DALIdentity {
            hash: "0".to_string(),
            state: 0,
            joined_on: current_blockstamp,
            penultimate_renewed_on: created_on,
            last_renewed_on: created_on,
            expires_on: created_time + super::constants::G1_PARAMS.ms_validity,
            revokes_on: created_time + super::constants::G1_PARAMS.ms_validity,
            expired_on: None,
            revoked_on: None,
            idty_doc: idty_doc.clone(),
        }
    }

    pub fn revoke_identity(
        db: &DuniterDB,
        wotb_id: NodeId,
        renewal_blockstamp: &Blockstamp,
        revert: bool,
    ) {
        let state = if revert { 2 } else { 1 };
        let revoked_on = if revert {
            String::from("")
        } else {
            renewal_blockstamp.to_string()
        };
        let mut cursor = db
            .0
            .prepare("UPDATE identities SET state=?, revoked_on=?  WHERE wotb_id=?;")
            .expect("Fail to exclude idty !")
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(state),
                sqlite::Value::String(revoked_on),
                sqlite::Value::Integer(wotb_id.0 as i64),
            ])
            .expect("Fail to exclude idty !");
    }

    pub fn renewal_identity(
        &mut self,
        db: &DuniterDB,
        pubkey: &ed25519::PublicKey,
        renewal_blockstamp: &Blockstamp,
        renawal_timestamp: u64,
        revert: bool,
    ) {
        let mut penultimate_renewed_block: Option<DALBlock> = None;
        let revert_excluding = if revert {
            penultimate_renewed_block = Some(
                DALBlock::get_block(self.idty_doc.currency(), db, &self.penultimate_renewed_on)
                    .expect("renewal_identity: Fail to get penultimate_renewed_block"),
            );
            penultimate_renewed_block.clone().unwrap().block.median_time
                + super::constants::G1_PARAMS.ms_validity < renawal_timestamp
        } else {
            false
        };
        self.state = if revert && revert_excluding { 1 } else { 0 };
        self.expires_on = if revert {
            penultimate_renewed_block.unwrap().block.median_time
                + super::constants::G1_PARAMS.ms_validity
        } else {
            renawal_timestamp + super::constants::G1_PARAMS.ms_validity
        };
        let mut cursor = db.0
            .prepare(
                "UPDATE identities SET state=?, last_renewed_on=?, expires_on=?, revokes_on=?  WHERE pubkey=?;",
            )
            .expect("Fail to renewal idty !")
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(self.state as i64),
                sqlite::Value::String(renewal_blockstamp.to_string()),
                sqlite::Value::Integer(self.expires_on as i64),
                sqlite::Value::Integer(
                    (renawal_timestamp + (super::constants::G1_PARAMS.ms_validity * 2)) as i64,
                ),
                sqlite::Value::String(pubkey.to_string()),
            ])
            .expect("Fail to renewal idty !");
    }

    pub fn remove_identity(db: &DuniterDB, wotb_id: NodeId) -> () {
        db.0
            .execute(format!(
                "DELETE FROM identities WHERE wotb_id={}",
                wotb_id.0
            ))
            .unwrap();
    }

    pub fn get_identity(
        currency: &str,
        db: &DuniterDB,
        pubkey: &ed25519::PublicKey,
    ) -> Option<DALIdentity> {
        let mut cursor = db
            .0
            .prepare(
                "SELECT uid, hash, sig,
                state, created_on, joined_on, penultimate_renewed_on, last_renewed_on,
                expires_on, revokes_on, expired_on, revoked_on FROM identities WHERE pubkey=?;",
            )
            .expect("Fail to get idty !")
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(pubkey.to_string())])
            .expect("Fail to get idty !");

        if let Some(row) = cursor.next().expect("get_identity: cursor error") {
            let idty_doc_builder = IdentityDocumentBuilder {
                currency,
                username: row[0]
                    .as_string()
                    .expect("get_identity: fail to parse username"),
                blockstamp: &Blockstamp::from_string(
                    row[4]
                        .as_string()
                        .expect("DB Error : idty created_on invalid !"),
                ).expect("DB Error : idty created_on invalid (2) !"),
                issuer: &pubkey,
            };
            let idty_sig = Signature::from_base64(
                row[2].as_string().expect("get_identity: fail to parse sig"),
            ).expect("get_identity: fail to parse sig (2)");
            let idty_doc = idty_doc_builder.build_with_signature(vec![idty_sig]);

            let expired_on = match Blockstamp::from_string(
                row[10]
                    .as_string()
                    .expect("get_identity: fail to parse expire on"),
            ) {
                Ok(blockstamp) => Some(blockstamp),
                Err(_) => None,
            };
            let revoked_on = match Blockstamp::from_string(
                row[11]
                    .as_string()
                    .expect("get_identity: fail to parse revoked on"),
            ) {
                Ok(blockstamp) => Some(blockstamp),
                Err(_) => None,
            };
            Some(DALIdentity {
                hash: row[2]
                    .as_string()
                    .expect("get_identity: fail to parse hash")
                    .to_string(),
                state: row[3]
                    .as_integer()
                    .expect("get_identity: fail to parse state") as isize,
                joined_on: Blockstamp::from_string(
                    row[5]
                        .as_string()
                        .expect("DB Error : idty joined_on invalid !"),
                ).expect("DB Error : idty joined_on invalid !"),
                penultimate_renewed_on: Blockstamp::from_string(
                    row[6]
                        .as_string()
                        .expect("DB Error : idty penultimate_renewed_on invalid !"),
                ).expect(
                    "DB Error : idty penultimate_renewed_on invalid (2) !",
                ),
                last_renewed_on: Blockstamp::from_string(
                    row[7]
                        .as_string()
                        .expect("get_identity: fail to parse last_renewed_on"),
                ).expect("get_identity: fail to parse last_renewed_on (2)"),
                expires_on: row[8]
                    .as_integer()
                    .expect("get_identity: fail to parse expires_on")
                    as u64,
                revokes_on: row[9]
                    .as_integer()
                    .expect("get_identity: fail to parse revokes_on")
                    as u64,
                expired_on,
                revoked_on,
                idty_doc,
            })
        } else {
            None
        }
    }
}

extern crate duniter_crypto;
extern crate duniter_documents;
extern crate duniter_wotb;
extern crate serde;
extern crate serde_json;
extern crate sqlite;

use self::duniter_crypto::keys;
use self::duniter_crypto::keys::{ed25519, PublicKey, Signature};
use self::duniter_documents::blockchain::v10::documents::identity::IdentityDocument;
use self::duniter_documents::blockchain::v10::documents::membership::MembershipType;
use self::duniter_documents::blockchain::v10::documents::BlockDocument;
use self::duniter_documents::blockchain::Document;
use self::duniter_documents::{BlockHash, BlockId, Blockstamp, Hash};
use self::duniter_wotb::NodeId;
use super::constants::MAX_FORKS;
use super::parsers::certifications::parse_certifications;
use super::parsers::excluded::parse_exclusions;
use super::parsers::identities::parse_identities;
use super::parsers::memberships::parse_memberships;
use super::parsers::revoked::parse_revocations;
use super::parsers::transactions::parse_compact_transactions;
use super::{DuniterDB, ForkState};
use std::collections::HashMap;

pub fn blockstamp_to_timestamp(blockstamp: &Blockstamp, db: &DuniterDB) -> Option<u64> {
    if blockstamp.id.0 == 0 {
        return Some(1_488_987_127);
    }
    let mut cursor = db
        .0
        .prepare("SELECT median_time FROM blocks WHERE number=? AND hash=? LIMIT 1;")
        .expect("convert blockstamp to timestamp failure at step 0 !")
        .cursor();

    cursor
        .bind(&[
            sqlite::Value::Integer(blockstamp.id.0 as i64),
            sqlite::Value::String(blockstamp.hash.0.to_hex()),
        ])
        .expect("convert blockstamp to timestamp failure at step 1 !");

    if let Some(row) = cursor
        .next()
        .expect("convert blockstamp to timestamp failure at step 2 !")
    {
        return Some(
            row[0]
                .as_integer()
                .expect("convert blockstamp to timestamp failure at step 3 !") as u64,
        );
    }
    None
}

#[derive(Debug, Copy, Clone)]
pub enum WotEvent {
    AddNode(ed25519::PublicKey, NodeId),
    RemNode(ed25519::PublicKey),
    AddLink(NodeId, NodeId),
    RemLink(NodeId, NodeId),
    EnableNode(NodeId),
    DisableNode(NodeId),
}

#[derive(Debug, Clone)]
pub struct BlockContext {
    pub blockstamp: Blockstamp,
    pub wot_events: Vec<WotEvent>,
}

#[derive(Debug, Clone)]
pub struct BlockContextV2 {
    pub blockstamp: Blockstamp,
    pub wot_events: Vec<WotEvent>,
}

#[derive(Debug, Clone)]
pub struct DALBlock {
    pub fork: usize,
    pub isolate: bool,
    pub block: BlockDocument,
}

impl DALBlock {
    pub fn blockstamp(&self) -> Blockstamp {
        self.block.blockstamp()
    }
}

pub fn get_forks(db: &DuniterDB) -> Vec<ForkState> {
    let mut forks = Vec::new();
    forks.push(ForkState::Full());
    for fork in 1..*MAX_FORKS {
        let mut cursor = db
            .0
            .prepare("SELECT isolate FROM blocks WHERE fork=? ORDER BY median_time DESC LIMIT 1;")
            .expect("Fail to get block !")
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(fork as i64)])
            .expect("Fail to get block !");

        if let Some(row) = cursor.next().unwrap() {
            if row[0].as_integer().unwrap() == 0 {
                forks.push(ForkState::Full())
            } else {
                forks.push(ForkState::Isolate())
            }
        } else {
            forks.push(ForkState::Free());
        }
    }
    forks
}

impl DALBlock {
    pub fn unisolate_fork(db: &DuniterDB, fork: usize) {
        db.0
            .execute(format!("UPDATE blocks SET isolate=0 WHERE fork={};", fork))
            .unwrap();
    }
    pub fn delete_fork(db: &DuniterDB, fork: usize) {
        db.0
            .execute(format!("DELETE FROM blocks WHERE fork={};", fork))
            .unwrap();
    }
    pub fn get_block_fork(db: &DuniterDB, blockstamp: &Blockstamp) -> Option<usize> {
        let mut cursor = db
            .0
            .prepare("SELECT fork FROM blocks WHERE number=? AND hash=?;")
            .expect("Fail to get block !")
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(blockstamp.id.0 as i64),
                sqlite::Value::String(blockstamp.hash.0.to_string()),
            ])
            .expect("Fail to get block !");

        if let Some(row) = cursor.next().unwrap() {
            Some(row[0].as_integer().unwrap() as usize)
        } else {
            None
        }
    }
    pub fn get_block_hash(db: &DuniterDB, block_number: &BlockId) -> Option<BlockHash> {
        let mut cursor = db
            .0
            .prepare("SELECT hash FROM blocks WHERE number=? AND fork=0;")
            .expect("Fail to get block !")
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(block_number.0 as i64)])
            .expect("Fail to get block !");

        if let Some(row) = cursor.next().unwrap() {
            Some(BlockHash(
                Hash::from_hex(row[0].as_string().unwrap()).unwrap(),
            ))
        } else {
            None
        }
    }

    pub fn get_blocks_hashs_all_forks(
        db: &DuniterDB,
        block_number: &BlockId,
    ) -> (Vec<BlockHash>, Vec<Hash>) {
        let mut cursor = db
            .0
            .prepare("SELECT hash, previous_hash FROM blocks WHERE number=?;")
            .expect("Fail to get block !")
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(block_number.0 as i64)])
            .expect("Fail to get block !");

        let mut hashs = Vec::new();
        let mut previous_hashs = Vec::new();
        while let Some(row) = cursor.next().unwrap() {
            hashs.push(BlockHash(
                Hash::from_hex(row[0].as_string().unwrap()).unwrap(),
            ));
            previous_hashs.push(Hash::from_hex(row[1].as_string().unwrap()).unwrap());
        }
        (hashs, previous_hashs)
    }

    pub fn get_stackables_blocks(
        currency: &str,
        db: &DuniterDB,
        current_blockstamp: &Blockstamp,
    ) -> Vec<DALBlock> {
        debug!("get_stackables_blocks() after {}", current_blockstamp);
        let mut stackables_blocks = Vec::new();
        let block_id = BlockId(current_blockstamp.id.0 + 1);
        let (hashs, previous_hashs) = DALBlock::get_blocks_hashs_all_forks(db, &block_id);
        for (hash, previous_hash) in hashs.into_iter().zip(previous_hashs) {
            if previous_hash == current_blockstamp.hash.0 {
                if let Some(dal_block) =
                    DALBlock::get_block(currency, db, &Blockstamp { id: block_id, hash })
                {
                    stackables_blocks.push(dal_block);
                } else {
                    panic!(format!(
                        "Fail to get stackable block {} !",
                        Blockstamp { id: block_id, hash }
                    ));
                }
            }
        }
        stackables_blocks
    }
    pub fn get_stackables_forks(db: &DuniterDB, current_blockstamp: &Blockstamp) -> Vec<usize> {
        let mut stackables_forks = Vec::new();
        let block_id = BlockId(current_blockstamp.id.0 + 1);
        let (hashs, previous_hashs) = DALBlock::get_blocks_hashs_all_forks(db, &block_id);
        for (hash, previous_hash) in hashs.into_iter().zip(previous_hashs) {
            if previous_hash == current_blockstamp.hash.0 {
                if let Some(fork) = DALBlock::get_block_fork(db, &Blockstamp { id: block_id, hash })
                {
                    if fork > 0 {
                        stackables_forks.push(fork);
                    }
                }
            }
        }
        stackables_forks
    }
    pub fn get_block(currency: &str, db: &DuniterDB, blockstamp: &Blockstamp) -> Option<DALBlock> {
        let mut cursor = db
            .0
            .prepare(
                "SELECT fork, isolate, nonce, number,
            pow_min, time, median_time, members_count,
            monetary_mass, unit_base, issuers_count, issuers_frame,
            issuers_frame_var, median_frame, second_tiercile_frame,
            currency, issuer, signature, hash, previous_hash, dividend, identities, joiners,
            actives, leavers, revoked, excluded, certifications,
            transactions FROM blocks WHERE number=? AND hash=?;",
            )
            .expect("Fail to get block !")
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(blockstamp.id.0 as i64),
                sqlite::Value::String(blockstamp.hash.0.to_string()),
            ])
            .expect("Fail to get block !");

        if let Some(row) = cursor.next().expect("block not found in bdd !") {
            let dividend_amount = row[20]
                .as_integer()
                .expect("dal::get_block() : fail to parse dividend !");
            let dividend = if dividend_amount > 0 {
                Some(dividend_amount as usize)
            } else if dividend_amount == 0 {
                None
            } else {
                return None;
            };
            let nonce = row[2]
                .as_integer()
                .expect("dal::get_block() : fail to parse nonce !") as u64;
            let inner_hash = Hash::from_hex(
                row[18]
                    .as_string()
                    .expect("dal::get_block() : fail to parse inner_hash !"),
            ).expect("dal::get_block() : fail to parse inner_hash (2) !");
            let identities = parse_identities(
                currency,
                row[21]
                    .as_string()
                    .expect("dal::get_block() : fail to parse identities !"),
            ).expect("dal::get_block() : fail to parse identities (2) !");
            let hashmap_identities = identities
                .iter()
                .map(|i| (i.issuers()[0], i.clone()))
                .collect::<HashMap<ed25519::PublicKey, IdentityDocument>>();
            Some(DALBlock {
                fork: row[0]
                    .as_integer()
                    .expect("dal::get_block() : fail to parse fork !")
                    as usize,
                isolate: if row[1]
                    .as_integer()
                    .expect("dal::get_block() : fail to parse isolate !")
                    == 0
                {
                    false
                } else {
                    true
                },
                block: BlockDocument {
                    nonce,
                    number: BlockId(
                        row[3]
                            .as_integer()
                            .expect("dal::get_block() : fail to parse number !")
                            as u32,
                    ),
                    pow_min: row[4]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse pow min !")
                        as usize,
                    time: row[5]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse time !")
                        as u64,
                    median_time: row[6]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse median_time !")
                        as u64,
                    members_count: row[7]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse members_count !")
                        as usize,
                    monetary_mass: row[8]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse monetary_mass !")
                        as usize,
                    unit_base: row[9]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse unit_base !")
                        as usize,
                    issuers_count: row[10]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse issuers_count !")
                        as usize,
                    issuers_frame: row[11]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse issuers_frame !")
                        as isize,
                    issuers_frame_var: row[12]
                        .as_integer()
                        .expect("dal::get_block() : fail to parse issuers_frame_var !")
                        as isize,
                    currency: row[15]
                        .as_string()
                        .expect("dal::get_block() : fail to parse currency !")
                        .to_string(),
                    issuers: vec![
                        PublicKey::from_base58(
                            row[16]
                                .as_string()
                                .expect("dal::get_block() : fail to parse issuer !"),
                        ).expect("dal::get_block() : fail to parse pubkey !"),
                    ],
                    signatures: vec![
                        Signature::from_base64(
                            row[17]
                                .as_string()
                                .expect("dal::get_block() : fail to parse signature !"),
                        ).expect("dal::get_block() : fail to parse signature (2) !"),
                    ],
                    hash: Some(BlockHash(
                        Hash::from_hex(
                            row[18]
                                .as_string()
                                .expect("dal::get_block() : fail to parse hash !"),
                        ).expect("dal::get_block() : fail to parse hash (2) !"),
                    )),
                    parameters: None,
                    previous_hash: Hash::from_hex(
                        row[19]
                            .as_string()
                            .expect("dal::get_block() : fail to parse previous_hash !"),
                    ).expect(
                        "dal::get_block() : fail to parse previous_hash (2) !",
                    ),
                    previous_issuer: None,
                    inner_hash: Some(inner_hash),
                    dividend,
                    identities: identities.clone(),
                    joiners: parse_memberships(
                        currency,
                        MembershipType::In(),
                        row[22]
                            .as_string()
                            .expect("dal::get_block() : fail to parse joiners !"),
                    ).expect("dal::get_block() : fail to parse joiners (2) !"),
                    actives: parse_memberships(
                        currency,
                        MembershipType::In(),
                        row[23]
                            .as_string()
                            .expect("dal::get_block() : fail to parse actives !"),
                    ).expect("dal::get_block() : fail to parse actives (2) !"),
                    leavers: parse_memberships(
                        currency,
                        MembershipType::Out(),
                        row[24]
                            .as_string()
                            .expect("dal::get_block() : fail to parse leavers !"),
                    ).expect("dal::get_block() : fail to parse leavers (2) !"),
                    revoked: parse_revocations(
                        currency,
                        db,
                        &hashmap_identities,
                        row[25]
                            .as_string()
                            .expect("dal::get_block() : fail to parse revoked !"),
                    ).expect("dal::get_block() : fail to parse revoked (2) !"),
                    excluded: parse_exclusions(
                        row[26]
                            .as_string()
                            .expect("dal::get_block() : fail to parse excluded !"),
                    ).expect("dal::get_block() : fail to parse excluded (2) !"),
                    certifications: parse_certifications(
                        currency,
                        db,
                        &hashmap_identities,
                        row[27]
                            .as_string()
                            .expect("dal::get_block() : fail to parse certifications !"),
                    ).expect(
                        "dal::get_block() : fail to parse certifications (2) !",
                    ),
                    transactions: parse_compact_transactions(
                        currency,
                        row[28]
                            .as_string()
                            .expect("dal::get_block() : fail to parse transactions !"),
                    ).expect("dal::get_block() : fail to parse transactions (2) !"),
                    inner_hash_and_nonce_str: format!(
                        "InnerHash: {}\nNonce: {}\n",
                        inner_hash.to_hex(),
                        nonce
                    ),
                },
                //median_frame: row[13].as_integer().unwrap_or(0) as usize,
                //second_tiercile_frame: row[14].as_integer().unwrap_or(0) as usize,
            })
        } else {
            None
        }
    }

    pub fn get_current_frame(&self, db: &DuniterDB) -> HashMap<keys::ed25519::PublicKey, usize> {
        let frame_begin = self.block.number.0 as i64 - (self.block.issuers_frame as i64);
        let mut current_frame: HashMap<keys::ed25519::PublicKey, usize> = HashMap::new();
        let mut cursor = db
            .0
            .prepare("SELECT issuer FROM blocks WHERE fork=0 AND number>=? LIMIT ?;")
            .expect("get current frame blocks failure at step 1 !")
            .cursor();
        cursor
            .bind(&[
                sqlite::Value::Integer(frame_begin),
                sqlite::Value::Integer(self.block.issuers_frame as i64),
            ])
            .expect("get current frame blocks failure at step 2 !");

        while let Some(row) = cursor
            .next()
            .expect("get current frame blocks failure at step 3 !")
        {
            let current_frame_copy = current_frame.clone();
            match current_frame_copy
                .get(&PublicKey::from_base58(row[0].as_string().unwrap()).unwrap())
            {
                Some(blocks_count) => {
                    if let Some(new_blocks_count) = current_frame
                        .get_mut(&PublicKey::from_base58(row[0].as_string().unwrap()).unwrap())
                    {
                        *new_blocks_count = *blocks_count + 1;
                    }
                }
                None => {
                    current_frame.insert(
                        PublicKey::from_base58(row[0].as_string().unwrap()).unwrap(),
                        0,
                    );
                }
            }
        }
        current_frame
    }

    pub fn compute_median_issuers_frame(&mut self, db: &DuniterDB) -> () {
        let current_frame = self.get_current_frame(db);
        if !current_frame.is_empty() {
            let mut current_frame_vec: Vec<_> = current_frame.values().cloned().collect();
            current_frame_vec.sort_unstable();

            /*// Calculate median
            let mut median_index = match self.block.issuers_count % 2 {
                1 => (self.block.issuers_count / 2) + 1,
                _ => self.block.issuers_count / 2,
            };
            if median_index >= self.block.issuers_count {
                median_index = self.block.issuers_count - 1;
            }
            self.median_frame = current_frame_vec[median_index];

            // Calculate second tiercile index
            let mut second_tiercile_index = match self.block.issuers_count % 3 {
                1 | 2 => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize + 1,
                _ => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize,
            };
            if second_tiercile_index >= self.block.issuers_count {
                second_tiercile_index = self.block.issuers_count - 1;
            }
            self.second_tiercile_frame = current_frame_vec[second_tiercile_index];*/
        }
    }
}

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

//! Sub-module that applies a block backwards.

use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait, BlockDocumentV10};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockNumber, Blockstamp};
use dubp_user_docs::documents::transaction::{TransactionDocument, TxAmount, TxBase};
use dup_crypto::keys::*;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_reader::indexes::sources::SourceAmount;
use durs_bc_db_writer::writers::requests::*;
use durs_bc_db_writer::{BinFreeStructDb, DbError};
use durs_common_tools::{fatal_error, UsizeSer32};
use durs_wot::data::{NewLinkResult, RemLinkResult};
use durs_wot::{WebOfTrust, WotId};
use std::collections::HashMap;
use unwrap::unwrap;

#[derive(Debug)]
/// Stores all queries to apply in database to "revert" the block
pub struct ValidBlockRevertReqs {
    pub new_current_blockstamp: Blockstamp,
    pub block_query: BlocksDBsWriteQuery,
    pub wot_queries: Vec<WotsDBsWriteQuery>,
    pub currency_queries: Vec<CurrencyDBsWriteQuery>,
}

#[derive(Debug)]
/// RevertValidBlockError
pub enum RevertValidBlockError {
    ExcludeUnknowNodeId(),
    RevokeUnknowNodeId(),
    DbError(DbError),
}

impl From<DbError> for RevertValidBlockError {
    fn from(e: DbError) -> Self {
        RevertValidBlockError::DbError(e)
    }
}

pub fn revert_block<W: WebOfTrust>(
    dal_block: BlockDb,
    wot_index: &mut HashMap<PubKey, WotId>,
    wot_db: &BinFreeStructDb<W>,
) -> Result<ValidBlockRevertReqs, RevertValidBlockError> {
    match dal_block.block {
        BlockDocument::V10(block_v10) => revert_block_v10(
            block_v10,
            unwrap!(dal_block.expire_certs),
            wot_index,
            wot_db,
        ),
    }
}

pub fn revert_block_v10<W: WebOfTrust>(
    mut block: BlockDocumentV10,
    expire_certs: HashMap<(WotId, WotId), BlockNumber>,
    wot_index: &mut HashMap<PubKey, WotId>,
    wot_db: &BinFreeStructDb<W>,
) -> Result<ValidBlockRevertReqs, RevertValidBlockError> {
    // Revert reduce block
    block.generate_inner_hash();
    debug!("blockchain: revert_valid_block({})", block.blockstamp());

    // REVERT_CURRENCY_EVENTS
    let mut currency_dbs_requests = Vec::new();
    // Revert transactions
    for tx_doc in block.transactions.iter().rev() {
        currency_dbs_requests.push(CurrencyDBsWriteQuery::RevertTx(Box::new(
            TransactionDocument::V10(tx_doc.clone()),
        )));
    }
    // Revert UD
    if let Some(UsizeSer32(du_amount)) = block.dividend {
        if du_amount > 0 {
            let members_wot_ids = wot_db
                .read(WebOfTrust::get_enabled)
                .expect("Fail to read WotDB");
            let mut members_pubkeys = Vec::new();
            for (pubkey, wot_id) in wot_index.iter() {
                if members_wot_ids.contains(wot_id) {
                    members_pubkeys.push(*pubkey);
                }
            }
            currency_dbs_requests.push(CurrencyDBsWriteQuery::RevertUD(
                SourceAmount(TxAmount(du_amount as isize), TxBase(block.unit_base.into())),
                block.number,
                members_pubkeys,
            ));
        }
    }
    // REVERT WOT EVENTS
    let mut wot_dbs_requests = Vec::new();
    // Revert expire_certs
    if !expire_certs.is_empty() {
        for ((source, target), created_block_id) in &expire_certs {
            wot_db
                .write(|db| {
                    let result = db.add_link(*source, *target);
                    match result {
                        NewLinkResult::Ok(_) => {}
                        _ => fatal_error!(
                            "Fail to add_link {}->{} : {:?}",
                            source.0,
                            target.0,
                            result
                        ),
                    }
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RevertExpireCert(
                *source,
                *target,
                *created_block_id,
            ));
        }
    }
    // Revert certifications
    for certification in block.certifications.clone() {
        trace!("stack_up_valid_block: apply cert...");
        let compact_cert = certification.to_compact_document();
        let wot_node_from = wot_index[&compact_cert.issuer];
        let wot_node_to = wot_index[&compact_cert.target];
        wot_db
            .write(|db| {
                let result = db.rem_link(wot_node_from, wot_node_to);
                match result {
                    RemLinkResult::Removed(_) => {}
                    _ => fatal_error!(
                        "Fail to rem_link {}->{} : {:?}",
                        wot_node_from.0,
                        wot_node_to.0,
                        result
                    ),
                }
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::RevertCert(
            compact_cert,
            wot_node_from,
            wot_node_to,
        ));
        trace!("stack_up_valid_block: apply cert...success.");
    }
    // Revert revocations
    for revocation in block.revoked.clone() {
        let compact_revoc = revocation.to_compact_document();
        let wot_id = if let Some(wot_id) = wot_index.get(&compact_revoc.issuer) {
            wot_id
        } else {
            return Err(RevertValidBlockError::RevokeUnknowNodeId());
        };
        wot_db
            .write(|db| {
                db.set_enabled(*wot_id, false);
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::RevertRevokeIdentity(
            compact_revoc.issuer,
            block.blockstamp(),
            true,
        ));
    }
    // Revert exclusions
    for exclusion in block.excluded.clone() {
        let wot_id = if let Some(wot_id) = wot_index.get(&exclusion) {
            wot_id
        } else {
            return Err(RevertValidBlockError::ExcludeUnknowNodeId());
        };
        wot_db
            .write(|db| {
                db.set_enabled(*wot_id, false);
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::RevertExcludeIdentity(
            exclusion,
            block.blockstamp(),
        ));
    }
    // List block identities
    let mut identities = HashMap::with_capacity(block.identities.len());
    for identity in block.identities.clone() {
        identities.insert(identity.issuers()[0], identity);
    }
    // Revert renewals
    for active in block.actives.clone() {
        let pubkey = active.issuers()[0];
        if !identities.contains_key(&pubkey) {
            let wot_id = wot_index[&pubkey];
            wot_db
                .write(|db| {
                    db.set_enabled(wot_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RevertRenewalIdentity(
                pubkey,
                wot_id,
                block.median_time,
                active.blockstamp().id,
            ));
        }
    }
    // Revert joiners
    for joiner in block.joiners.iter().rev() {
        let pubkey = joiner.clone().issuers()[0];
        if let Some(_idty_doc) = identities.get(&pubkey) {
            // Newcomer
            wot_db
                .write(|db| {
                    db.rem_node();
                })
                .expect("Fail to write in WotDB");
            wot_index.remove(&pubkey);
            wot_dbs_requests.push(WotsDBsWriteQuery::RevertCreateIdentity(pubkey));
        } else {
            // Renewer
            let wot_id = wot_index[&joiner.issuers()[0]];
            wot_db
                .write(|db| {
                    db.set_enabled(wot_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RevertRenewalIdentity(
                joiner.issuers()[0],
                wot_id,
                block.common_time(),
                joiner.blockstamp().id,
            ));
        }
    }
    // Return DBs requests
    Ok(ValidBlockRevertReqs {
        new_current_blockstamp: block.previous_blockstamp(),
        block_query: BlocksDBsWriteQuery::RevertBlock(BlockDb {
            block: BlockDocument::V10(block),
            expire_certs: Some(expire_certs),
        }),
        wot_queries: wot_dbs_requests,
        currency_queries: currency_dbs_requests,
    })
}

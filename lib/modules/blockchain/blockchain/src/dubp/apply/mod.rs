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

//! Sub-module that applies the content of a block to the indexes of the local blockchain.

use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait, BlockDocumentV10};
use dubp_common_doc::traits::Document;
use dubp_common_doc::BlockNumber;
use dubp_user_docs::documents::transaction::{TxAmount, TxBase};
use dup_crypto::keys::*;
use durs_bc_db_reader::entities::block::DbBlock;
use durs_bc_db_reader::entities::sources::SourceAmount;
use durs_bc_db_writer::writers::requests::*;
use durs_bc_db_writer::BinFreeStructDb;
use durs_common_tools::fatal_error;
use durs_wot::data::NewLinkResult;
use durs_wot::{WebOfTrust, WotId};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
/// Stores all queries to apply in database to "apply" the block
pub struct ValidBlockApplyReqs(
    pub BlocksDBsWriteQuery,
    pub Vec<WotsDBsWriteQuery>,
    pub Vec<CurrencyDBsWriteQuery>,
);

#[derive(Debug, Copy, Clone)]
/// ApplyValidBlockError
pub enum ApplyValidBlockError {
    DBsCorrupted,
    ExcludeUnknowNodeId,
    RevokeUnknowNodeId,
}

#[inline]
pub fn apply_valid_block<W: WebOfTrust>(
    block: BlockDocument,
    wot_index: &mut HashMap<PubKey, WotId>,
    wot_db: &BinFreeStructDb<W>,
    expire_certs: &HashMap<(WotId, WotId), BlockNumber>,
) -> Result<ValidBlockApplyReqs, ApplyValidBlockError> {
    match block {
        BlockDocument::V10(block_v10) => {
            apply_valid_block_v10(block_v10, wot_index, wot_db, expire_certs)
        }
    }
}

pub fn apply_valid_block_v10<W: WebOfTrust>(
    mut block: BlockDocumentV10,
    wot_index: &mut HashMap<PubKey, WotId>,
    wot_db: &BinFreeStructDb<W>,
    expire_certs: &HashMap<(WotId, WotId), BlockNumber>,
) -> Result<ValidBlockApplyReqs, ApplyValidBlockError> {
    debug!(
        "BlockchainModule : apply_valid_block({})",
        block.blockstamp(),
    );
    let mut wot_dbs_requests = Vec::new();
    let mut currency_dbs_requests = Vec::new();
    let current_blockstamp = block.blockstamp();
    let mut identities = HashMap::with_capacity(block.identities.len());
    for identity in &block.identities {
        identities.insert(identity.issuers()[0], identity);
    }
    for joiner in &block.joiners {
        let pubkey = joiner.issuers()[0];
        if let Some(idty_doc) = identities.get(&pubkey) {
            // Newcomer
            let wot_id = WotId(
                wot_db
                    .read(WebOfTrust::size)
                    .expect("Fatal error : fail to read WotDB !"),
            );
            wot_db
                .write(|db| {
                    db.add_node();
                })
                .expect("Fail to write in WotDB");
            wot_index.insert(pubkey, wot_id);
            wot_dbs_requests.push(WotsDBsWriteQuery::CreateIdentity(
                wot_id,
                current_blockstamp,
                block.median_time,
                Box::new((*idty_doc).clone()),
                joiner.blockstamp().id,
            ));
        } else {
            // Renewer
            let wot_id = wot_index[&joiner.issuers()[0]];
            wot_db
                .write(|db| {
                    db.set_enabled(wot_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RenewalIdentity(
                joiner.issuers()[0],
                wot_id,
                block.median_time,
                joiner.blockstamp().id,
            ));
        }
    }
    for active in &block.actives {
        let pubkey = active.issuers()[0];
        if !identities.contains_key(&pubkey) {
            let wot_id = wot_index[&pubkey];
            wot_db
                .write(|db| {
                    db.set_enabled(wot_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RenewalIdentity(
                pubkey,
                wot_id,
                block.median_time,
                active.blockstamp().id,
            ));
        }
    }
    for exclusion in &block.excluded {
        let wot_id = if let Some(wot_id) = wot_index.get(&exclusion) {
            wot_id
        } else {
            return Err(ApplyValidBlockError::ExcludeUnknowNodeId);
        };
        wot_db
            .write(|db| {
                db.set_enabled(*wot_id, false);
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::ExcludeIdentity(
            *exclusion,
            block.blockstamp(),
        ));
    }
    for revocation in &block.revoked {
        let compact_revoc = revocation.to_compact_document();
        let wot_id = if let Some(wot_id) = wot_index.get(&compact_revoc.issuer) {
            wot_id
        } else {
            return Err(ApplyValidBlockError::RevokeUnknowNodeId);
        };
        wot_db
            .write(|db| {
                db.set_enabled(*wot_id, false);
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::RevokeIdentity(
            compact_revoc.issuer,
            block.blockstamp(),
            true,
        ));
    }
    for certification in &block.certifications {
        trace!("stack_up_valid_block: apply cert...");
        let compact_cert = certification.to_compact_document();
        let wot_node_from = wot_index
            .get(&compact_cert.issuer)
            .ok_or(ApplyValidBlockError::DBsCorrupted)?;
        let wot_node_to = wot_index
            .get(&compact_cert.target)
            .ok_or(ApplyValidBlockError::DBsCorrupted)?;
        wot_db
            .write(|db| {
                let result = db.add_link(*wot_node_from, *wot_node_to);
                match result {
                    NewLinkResult::Ok(_) => {}
                    _ => fatal_error!(
                        "Fail to add_link {}->{} : {:?}",
                        wot_node_from.0,
                        wot_node_to.0,
                        result
                    ),
                }
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::CreateCert(
            compact_cert.issuer,
            *wot_node_from,
            *wot_node_to,
            compact_cert.block_number,
            block.median_time,
        ));
        trace!("stack_up_valid_block: apply cert...success.");
    }
    if !expire_certs.is_empty() {
        let mut blocks_already_expire = HashSet::new();
        for ((source, target), created_block_id) in expire_certs {
            if !blocks_already_expire.contains(created_block_id) {
                wot_dbs_requests.push(WotsDBsWriteQuery::ExpireCerts(*created_block_id));
                blocks_already_expire.insert(*created_block_id);
            }
            wot_db
                .write(|db| {
                    let _ = db.rem_link(*source, *target);
                })
                .expect("Fail to write in WotDB");
        }
    }
    if let Some(du_amount) = block.dividend {
        if du_amount > 0 {
            let members_wot_ids = wot_db
                .read(WebOfTrust::get_enabled)
                .expect("Fail to read WotDB");
            let mut members_pubkeys = Vec::new();
            for (pubkey, wot_id) in wot_index {
                if members_wot_ids.contains(wot_id) {
                    members_pubkeys.push(*pubkey);
                }
            }
            currency_dbs_requests.push(CurrencyDBsWriteQuery::CreateUD(
                SourceAmount(TxAmount(du_amount as isize), TxBase(block.unit_base)),
                block.number,
                members_pubkeys,
            ));
        }
    }

    for tx in &block.transactions {
        currency_dbs_requests.push(CurrencyDBsWriteQuery::WriteTx(Box::new(tx.unwrap_doc())));
    }

    /*// Calculate the state of the wot
    if !wot_events.is_empty() && verif_level != SyncVerificationLevel::FastSync() {
        // Calculate sentries_count
        let sentries_count = wot.get_sentries(3).len();
        // Calculate average_density
        let average_density = calculate_average_density::<W>(&wot);
        let sentry_requirement =
            get_sentry_requirement(block.members_count, G1_PARAMS.step_max);
        // Calculate distances and connectivities
        let (average_distance, distances, average_connectivity, connectivities) =
            compute_distances::<W>(
                &wot,
                sentry_requirement,
                G1_PARAMS.step_max,
                G1_PARAMS.x_percent,
            );
        // Calculate centralities and average_centrality
        let centralities =
            calculate_distance_stress_centralities::<W>(&wot, G1_PARAMS.step_max);
        let average_centrality =
            (centralities.iter().sum::<u64>() as f64 / centralities.len() as f64) as usize;
        // Register the state of the wot
        let max_connectivity = currency_params.max_connectivity();
        durs_bc_db_writer::register_wot_state(
            db,
            &WotState {
                block_number: block.number.0,
                block_hash: block.hash.expect("Fail to get block hash").to_string(),
                sentries_count,
                average_density,
                average_distance,
                distances,
                average_connectivity,
                connectivities: connectivities
                    .iter()
                    .map(|c| {
                        if *c > max_connectivity {
                            max_connectivity
                        } else {
                            *c
                        }
                    })
                    .collect(),
                average_centrality,
                centralities,
            },
        );
    }*/
    // Create DbBlock
    block.reduce();
    let dal_block = DbBlock {
        block: BlockDocument::V10(block),
        expire_certs: Some(expire_certs.clone()),
    };
    // Return DBs requests
    Ok(ValidBlockApplyReqs(
        BlocksDBsWriteQuery::WriteBlock(dal_block),
        wot_dbs_requests,
        currency_dbs_requests,
    ))
}

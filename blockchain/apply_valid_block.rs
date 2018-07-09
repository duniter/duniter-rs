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
use duniter_dal::block::DALBlock;
use duniter_dal::sources::SourceAmount;
use duniter_dal::writers::requests::*;
use duniter_dal::{BinDB, ForkId};
use duniter_documents::blockchain::v10::documents::transaction::{TxAmount, TxBase};
use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::blockchain::Document;
use duniter_documents::BlockId;
use duniter_wotb::data::{NewLinkResult, RemLinkResult};
use duniter_wotb::{NodeId, WebOfTrust};
use rustbreak::backend::Backend;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[derive(Debug)]
/// Stores all queries to apply in database to "apply" the block
pub struct ValidBlockApplyReqs(
    pub BlocksDBsWriteQuery,
    pub Vec<WotsDBsWriteQuery>,
    pub Vec<CurrencyDBsWriteQuery>,
);

#[derive(Debug, Copy, Clone)]
/// ApplyValidBlockError
pub enum ApplyValidBlockError {
    ExcludeUnknowNodeId(),
    RevokeUnknowNodeId(),
}

pub fn apply_valid_block<W: WebOfTrust, B: Backend + Debug>(
    block: &BlockDocument,
    wot_index: &mut HashMap<PubKey, NodeId>,
    wot_db: &BinDB<W, B>,
    expire_certs: &HashMap<(NodeId, NodeId), BlockId>,
    old_fork_id: Option<ForkId>,
) -> Result<ValidBlockApplyReqs, ApplyValidBlockError> {
    debug!(
        "BlockchainModule : apply_valid_block({})",
        block.blockstamp()
    );
    let mut wot_dbs_requests = Vec::new();
    let mut currency_dbs_requests = Vec::new();
    let current_blockstamp = block.blockstamp();
    let mut identities = HashMap::with_capacity(block.identities.len());
    for identity in block.identities.clone() {
        identities.insert(identity.issuers()[0], identity);
    }
    for joiner in block.joiners.clone() {
        let pubkey = joiner.clone().issuers()[0];
        if let Some(idty_doc) = identities.get(&pubkey) {
            // Newcomer
            let wotb_id = NodeId(
                wot_db
                    .read(|db| db.size())
                    .expect("Fatal error : fail to read WotDB !"),
            );
            wot_db
                .write(|db| {
                    db.add_node();
                })
                .expect("Fail to write in WotDB");
            wot_index.insert(pubkey, wotb_id);
            wot_dbs_requests.push(WotsDBsWriteQuery::CreateIdentity(
                wotb_id,
                current_blockstamp,
                block.median_time,
                Box::new(idty_doc.clone()),
                joiner.blockstamp().id,
            ));
        } else {
            // Renewer
            let wotb_id = wot_index[&joiner.issuers()[0]];
            wot_db
                .write(|db| {
                    db.set_enabled(wotb_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RenewalIdentity(
                joiner.issuers()[0],
                wotb_id,
                block.median_time,
                joiner.blockstamp().id,
            ));
        }
    }
    for active in block.actives.clone() {
        let pubkey = active.issuers()[0];
        if !identities.contains_key(&pubkey) {
            let wotb_id = wot_index[&pubkey];
            wot_db
                .write(|db| {
                    db.set_enabled(wotb_id, true);
                })
                .expect("Fail to write in WotDB");
            wot_dbs_requests.push(WotsDBsWriteQuery::RenewalIdentity(
                pubkey,
                wotb_id,
                block.median_time,
                active.blockstamp().id,
            ));
        }
    }
    for exclusion in block.excluded.clone() {
        let wot_id = if let Some(wot_id) = wot_index.get(&exclusion) {
            wot_id
        } else {
            return Err(ApplyValidBlockError::ExcludeUnknowNodeId());
        };
        wot_db
            .write(|db| {
                db.set_enabled(*wot_id, false);
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::ExcludeIdentity(
            exclusion,
            block.blockstamp(),
        ));
    }
    for revocation in block.revoked.clone() {
        let compact_revoc = revocation.to_compact_document();
        let wot_id = if let Some(wot_id) = wot_index.get(&compact_revoc.issuer) {
            wot_id
        } else {
            return Err(ApplyValidBlockError::RevokeUnknowNodeId());
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
    for certification in block.certifications.clone() {
        trace!("stack_up_valid_block: apply cert...");
        let compact_cert = certification.to_compact_document();
        let wotb_node_from = wot_index[&compact_cert.issuer];
        let wotb_node_to = wot_index[&compact_cert.target];
        wot_db
            .write(|db| {
                let result = db.add_link(wotb_node_from, wotb_node_to);
                match result {
                    NewLinkResult::Ok(_) => {}
                    _ => panic!(
                        "Fail to add_link {}->{} : {:?}",
                        wotb_node_from.0, wotb_node_to.0, result
                    ),
                }
            })
            .expect("Fail to write in WotDB");
        wot_dbs_requests.push(WotsDBsWriteQuery::CreateCert(
            compact_cert.issuer,
            wotb_node_from,
            wotb_node_to,
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
                    let result = db.rem_link(*source, *target);
                    match result {
                        RemLinkResult::Removed(_) => {}
                        _ => panic!("Fail to rem_link {}->{} : {:?}", source.0, target.0, result),
                    }
                })
                .expect("Fail to write in WotDB");
        }
    }
    if let Some(du_amount) = block.dividend {
        if du_amount > 0 {
            let members_wot_ids = wot_db
                .read(|db| db.get_enabled())
                .expect("Fail to read WotDB");
            let mut members_pubkeys = Vec::new();
            for (pubkey, wotb_id) in wot_index {
                if members_wot_ids.contains(wotb_id) {
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
    for tx in block.transactions.clone() {
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
            duniter_dal::register_wot_state(
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
    // Create DALBlock
    let mut block = block.clone();
    let previous_blockcstamp = block.previous_blockstamp();
    let block_hash = block
        .hash
        .expect("Try to get hash of an uncompleted or reduce block !");
    block.reduce();
    let dal_block = DALBlock {
        block,
        fork_id: ForkId(0),
        isolate: false,
        expire_certs: Some(expire_certs.clone()),
    };
    // Return DBs requests
    Ok(ValidBlockApplyReqs(
        BlocksDBsWriteQuery::WriteBlock(
            Box::new(dal_block),
            old_fork_id,
            previous_blockcstamp,
            block_hash,
        ),
        wot_dbs_requests,
        currency_dbs_requests,
    ))
}

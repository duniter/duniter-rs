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

extern crate duniter_crypto;
extern crate duniter_dal;
extern crate duniter_documents;
extern crate duniter_wotb;

use duniter_crypto::keys::*;
use duniter_dal::block::{DALBlock, WotEvent};
use duniter_dal::writers::requests::DBWriteRequest;
use duniter_documents::blockchain::v10::documents::BlockDocument;
use duniter_documents::blockchain::Document;
use duniter_wotb::{NodeId, WebOfTrust};

use std::collections::HashMap;

pub fn try_stack_up_completed_block<W: WebOfTrust + Sync>(
    block: &BlockDocument,
    wotb_index: &HashMap<PubKey, NodeId>,
    wot: &W,
) -> (bool, Vec<DBWriteRequest>, Vec<WotEvent>) {
    debug!(
        "BlockchainModule : try stack up complete block {}",
        block.blockstamp()
    );
    let mut db_requests = Vec::new();
    let mut wot_events = Vec::new();
    let mut wot_copy: W = wot.clone();
    let mut wotb_index_copy: HashMap<PubKey, NodeId> = wotb_index.clone();
    let current_blockstamp = block.blockstamp();
    let mut identities = HashMap::with_capacity(block.identities.len());
    for identity in block.identities.clone() {
        identities.insert(identity.issuers()[0], identity);
    }
    for joiner in block.joiners.clone() {
        let pubkey = joiner.clone().issuers()[0];
        if let Some(idty_doc) = identities.get(&pubkey) {
            // Newcomer
            let wotb_id = NodeId(wot_copy.size());
            wot_events.push(WotEvent::AddNode(pubkey, wotb_id));
            wot_copy.add_node();
            wotb_index_copy.insert(pubkey, wotb_id);
            db_requests.push(DBWriteRequest::CreateIdentity(
                wotb_id,
                current_blockstamp,
                block.median_time,
                Box::new(idty_doc.clone()),
            ));
        } else {
            // Renewer
            let wotb_id = wotb_index_copy[&joiner.issuers()[0]];
            wot_events.push(WotEvent::EnableNode(wotb_id));
            wot_copy.set_enabled(wotb_id, true);
            db_requests.push(DBWriteRequest::RenewalIdentity(
                joiner.issuers()[0],
                block.blockstamp(),
                block.median_time,
            ));
        }
    }
    for active in block.actives.clone() {
        let pubkey = active.issuers()[0];
        if !identities.contains_key(&pubkey) {
            let wotb_id = wotb_index_copy[&pubkey];
            wot_events.push(WotEvent::EnableNode(wotb_id));
            wot_copy.set_enabled(wotb_id, true);
            db_requests.push(DBWriteRequest::RenewalIdentity(
                pubkey,
                block.blockstamp(),
                block.median_time,
            ));
        }
    }
    for exclusion in block.excluded.clone() {
        let wotb_id = wotb_index_copy[&exclusion];
        wot_events.push(WotEvent::DisableNode(wotb_id));
        wot_copy.set_enabled(wotb_id, false);
        db_requests.push(DBWriteRequest::ExcludeIdentity(
            wotb_id,
            block.blockstamp(),
            block.median_time,
        ));
    }
    for revocation in block.revoked.clone() {
        let compact_revoc = revocation.to_compact_document();
        let wotb_id = wotb_index_copy[&compact_revoc.issuer];
        wot_events.push(WotEvent::DisableNode(wotb_id));
        wot_copy.set_enabled(wotb_id, false);
        db_requests.push(DBWriteRequest::RevokeIdentity(
            wotb_id,
            block.blockstamp(),
            block.median_time,
        ));
    }
    for certification in block.certifications.clone() {
        trace!("try_stack_up_completed_block: apply cert...");
        let compact_cert = certification.to_compact_document();
        let wotb_node_from = wotb_index_copy[&compact_cert.issuer];
        let wotb_node_to = wotb_index_copy[&compact_cert.target];
        wot_events.push(WotEvent::AddLink(wotb_node_from, wotb_node_to));
        wot_copy.add_link(wotb_node_from, wotb_node_to);
        db_requests.push(DBWriteRequest::CreateCert(
            block.blockstamp(),
            block.median_time,
            compact_cert,
        ));
        trace!("try_stack_up_completed_block: apply cert...success.");
    }

    /*// Calculate the state of the wot
        if !wot_events.is_empty() && verif_level != SyncVerificationLevel::FastSync() {
            // Calculate sentries_count
            let sentries_count = wot_copy.get_sentries(3).len();
            // Calculate average_density
            let average_density = calculate_average_density::<W>(&wot_copy);
            let sentry_requirement =
                get_sentry_requirement(block.members_count, G1_PARAMS.step_max);
            // Calculate distances and connectivities
            let (average_distance, distances, average_connectivity, connectivities) =
                compute_distances::<W>(
                    &wot_copy,
                    sentry_requirement,
                    G1_PARAMS.step_max,
                    G1_PARAMS.x_percent,
                );
            // Calculate centralities and average_centrality
            let centralities =
                calculate_distance_stress_centralities::<W>(&wot_copy, G1_PARAMS.step_max);
            let average_centrality =
                (centralities.iter().sum::<u64>() as f64 / centralities.len() as f64) as usize;
            // Register the state of the wot
            duniter_dal::register_wot_state(
                db,
                &WotState {
                    block_number: block.number.0,
                    block_hash: block.hash.unwrap().to_string(),
                    sentries_count,
                    average_density,
                    average_distance,
                    distances,
                    average_connectivity,
                    connectivities: connectivities
                        .iter()
                        .map(|c| {
                            if *c > *G1_CONNECTIVITY_MAX {
                                *G1_CONNECTIVITY_MAX
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
    // Write block in bdd
    db_requests.push(DBWriteRequest::WriteBlock(Box::new(DALBlock {
        block: block.clone(),
        fork: 0,
        isolate: false,
    })));

    (true, db_requests, wot_events)
}

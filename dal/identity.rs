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

use currency_params::CurrencyParameters;
use duniter_crypto::keys::*;
use duniter_documents::v10::identity::IdentityDocument;
use duniter_documents::{BlockId, Blockstamp};
use durs_wot::NodeId;
use std::collections::HashMap;
use {BinDB, DALError, IdentitiesV10Datas, MsExpirV10Datas};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Identity state
pub enum DALIdentityState {
    /// Member
    Member(Vec<usize>),
    /// Expire Member
    ExpireMember(Vec<usize>),
    /// Explicit Revoked
    ExplicitRevoked(Vec<usize>),
    /// Explicit Revoked after expire
    ExplicitExpireRevoked(Vec<usize>),
    /// Implicit revoked
    ImplicitRevoked(Vec<usize>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Identity in database
pub struct DALIdentity {
    /// Identity hash
    pub hash: String,
    /// Identity state
    pub state: DALIdentityState,
    /// Blockstamp the identity was written
    pub joined_on: Blockstamp,
    /// Blockstamp the identity was expired
    pub expired_on: Option<Blockstamp>,
    /// Blockstamp the identity was revoked
    pub revoked_on: Option<Blockstamp>,
    /// Identity document
    pub idty_doc: IdentityDocument,
    /// Identity wot id
    pub wot_id: NodeId,
    /// Membership created block number
    pub ms_created_block_id: BlockId,
    /// Timestamp from which membership can be renewed
    pub ms_chainable_on: Vec<u64>,
    /// Timestamp from which the identity can write a new certification
    pub cert_chainable_on: Vec<u64>,
}

/// Get uid from pubkey
pub fn get_uid(
    identities_db: &BinDB<IdentitiesV10Datas>,
    pubkey: PubKey,
) -> Result<Option<String>, DALError> {
    Ok(identities_db.read(|db| {
        if let Some(dal_idty) = db.get(&pubkey) {
            Some(String::from(dal_idty.idty_doc.username()))
        } else {
            None
        }
    })?)
}

/// Get pubkey from uid
pub fn get_pubkey_from_uid(
    identities_db: &BinDB<IdentitiesV10Datas>,
    uid: &str,
) -> Result<Option<PubKey>, DALError> {
    Ok(identities_db.read(|db| {
        for (pubkey, dal_idty) in db {
            if uid == dal_idty.idty_doc.username() {
                return Some(*pubkey);
            }
        }
        None
    })?)
}

impl DALIdentity {
    /// Apply "exclude identity" event
    pub fn exclude_identity(
        identities_db: &BinDB<IdentitiesV10Datas>,
        pubkey: &PubKey,
        exclusion_blockstamp: &Blockstamp,
        revert: bool,
    ) -> Result<(), DALError> {
        let mut idty_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : try to renewal unknow identity !");
        idty_datas.state = if revert {
            match idty_datas.state {
                DALIdentityState::ExpireMember(renewed_counts) => {
                    DALIdentityState::Member(renewed_counts)
                }
                _ => panic!("Try to revert exclusion for a no excluded identity !"),
            }
        } else {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    DALIdentityState::ExpireMember(renewed_counts)
                }
                _ => panic!("Try to exclude for an already excluded/revoked identity !"),
            }
        };
        idty_datas.expired_on = if revert {
            None
        } else {
            Some(*exclusion_blockstamp)
        };
        // Write new identity datas
        identities_db.write(|db| {
            db.insert(*pubkey, idty_datas);
        })?;
        Ok(())
    }

    /// Get wot_id index
    pub fn get_wot_index(
        identities_db: &BinDB<IdentitiesV10Datas>,
    ) -> Result<HashMap<PubKey, NodeId>, DALError> {
        Ok(identities_db.read(|db| {
            let mut wot_index: HashMap<PubKey, NodeId> = HashMap::new();
            for (pubkey, member_datas) in db {
                let wot_id = member_datas.wot_id;
                wot_index.insert(*pubkey, wot_id);
            }
            wot_index
        })?)
    }

    /// Apply "revoke identity" event
    pub fn revoke_identity(
        identities_db: &BinDB<IdentitiesV10Datas>,
        pubkey: &PubKey,
        renewal_blockstamp: &Blockstamp,
        explicit: bool,
        revert: bool,
    ) -> Result<(), DALError> {
        let mut member_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : Try to revoke unknow idty !");

        member_datas.state = if revert {
            match member_datas.state {
                DALIdentityState::ExplicitRevoked(renewed_counts) => {
                    DALIdentityState::Member(renewed_counts)
                }
                DALIdentityState::ExplicitExpireRevoked(renewed_counts)
                | DALIdentityState::ImplicitRevoked(renewed_counts) => {
                    DALIdentityState::ExpireMember(renewed_counts)
                }
                _ => panic!("Try to revert revoke_identity() for a no revoked idty !"),
            }
        } else {
            match member_datas.state {
                DALIdentityState::ExpireMember(renewed_counts) => {
                    DALIdentityState::ExplicitExpireRevoked(renewed_counts)
                }
                DALIdentityState::Member(renewed_counts) => {
                    if explicit {
                        DALIdentityState::ExplicitRevoked(renewed_counts)
                    } else {
                        DALIdentityState::ImplicitRevoked(renewed_counts)
                    }
                }
                _ => panic!("Try to revert revoke an already revoked idty !"),
            }
        };
        member_datas.revoked_on = if revert {
            None
        } else {
            Some(*renewal_blockstamp)
        };

        identities_db.write(|db| {
            db.insert(*pubkey, member_datas);
        })?;
        Ok(())
    }

    /// Apply "renewal identity" event in databases
    pub fn renewal_identity(
        &mut self,
        currency_params: &CurrencyParameters,
        identities_db: &BinDB<IdentitiesV10Datas>,
        ms_db: &BinDB<MsExpirV10Datas>,
        pubkey: &PubKey,
        idty_wot_id: NodeId,
        renewal_timestamp: u64,
        ms_created_block_id: BlockId,
        revert: bool,
    ) -> Result<(), DALError> {
        // Get idty_datas
        let mut idty_datas = identities_db
            .read(|db| db.get(pubkey).cloned())?
            .expect("Fatal error : try to renewal unknow identity !");
        // Calculate new state value
        idty_datas.state = if revert {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts[renewed_counts.len() - 1] -= 1;
                    if new_renewed_counts[renewed_counts.len() - 1] > 0 {
                        DALIdentityState::Member(new_renewed_counts)
                    } else {
                        DALIdentityState::ExpireMember(new_renewed_counts)
                    }
                }
                _ => panic!("Try to revert renewal_identity() for an excluded or revoked idty !"),
            }
        } else {
            match idty_datas.state {
                DALIdentityState::Member(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts[renewed_counts.len() - 1] += 1;
                    DALIdentityState::Member(new_renewed_counts)
                }
                DALIdentityState::ExpireMember(renewed_counts) => {
                    let mut new_renewed_counts = renewed_counts.clone();
                    new_renewed_counts.push(0);
                    DALIdentityState::Member(new_renewed_counts)
                }
                _ => panic!("Try to renewed a revoked identity !"),
            }
        };
        // Calculate new ms_chainable_on value
        if revert {
            idty_datas.ms_chainable_on.pop();
        } else {
            idty_datas
                .ms_chainable_on
                .push(renewal_timestamp + currency_params.ms_period);
        }
        // Write new identity datas
        identities_db.write(|db| {
            db.insert(*pubkey, idty_datas);
        })?;
        // Update MsExpirV10DB
        ms_db.write(|db| {
            let mut memberships = db.get(&ms_created_block_id).cloned().unwrap_or_default();
            memberships.insert(idty_wot_id);
            db.insert(ms_created_block_id, memberships);
        })?;
        Ok(())
    }

    /// Remove identity from databases
    pub fn remove_identity(db: &BinDB<IdentitiesV10Datas>, pubkey: PubKey) -> Result<(), DALError> {
        db.write(|db| {
            db.remove(&pubkey);
        })?;
        Ok(())
    }

    /// Get identity in databases
    pub fn get_identity(
        db: &BinDB<IdentitiesV10Datas>,
        pubkey: &PubKey,
    ) -> Result<Option<DALIdentity>, DALError> {
        Ok(db.read(|db| {
            if let Some(member_datas) = db.get(&pubkey) {
                Some(member_datas.clone())
            } else {
                None
            }
        })?)
    }
}

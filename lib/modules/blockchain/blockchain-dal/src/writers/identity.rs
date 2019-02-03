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

use crate::entities::currency_params::CurrencyParameters;
use crate::entities::identity::{DALIdentity, DALIdentityState};
use crate::{BinDB, DALError, IdentitiesV10Datas, MsExpirV10Datas};
use dubp_documents::documents::identity::IdentityDocument;
use dubp_documents::Document;
use dubp_documents::{BlockId, Blockstamp};
use dup_crypto::keys::PubKey;
use durs_wot::NodeId;

/// Remove identity from databases
pub fn revert_create_identity(
    identities_db: &BinDB<IdentitiesV10Datas>,
    ms_db: &BinDB<MsExpirV10Datas>,
    pubkey: &PubKey,
) -> Result<(), DALError> {
    let dal_idty = identities_db.read(|db| {
        db.get(&pubkey)
            .expect("Fatal error : try to revert unknow identity !")
            .clone()
    })?;
    // Remove membership
    ms_db.write(|db| {
        let mut memberships = db
            .get(&dal_idty.ms_created_block_id)
            .cloned()
            .expect("Try to revert a membership that does not exist !");
        memberships.remove(&dal_idty.wot_id);
        db.insert(dal_idty.ms_created_block_id, memberships);
    })?;
    // Remove identity
    identities_db.write(|db| {
        db.remove(&dal_idty.idty_doc.issuers()[0]);
    })?;
    Ok(())
}

/// Write identity in databases
pub fn create_identity(
    currency_params: &CurrencyParameters,
    identities_db: &BinDB<IdentitiesV10Datas>,
    ms_db: &BinDB<MsExpirV10Datas>,
    idty_doc: &IdentityDocument,
    ms_created_block_id: BlockId,
    wot_id: NodeId,
    current_blockstamp: Blockstamp,
    current_bc_time: u64,
) -> Result<(), DALError> {
    let mut idty_doc = idty_doc.clone();
    idty_doc.reduce();
    let idty = DALIdentity {
        hash: "0".to_string(),
        state: DALIdentityState::Member(vec![0]),
        joined_on: current_blockstamp,
        expired_on: None,
        revoked_on: None,
        idty_doc,
        wot_id,
        ms_created_block_id,
        ms_chainable_on: vec![current_bc_time + currency_params.ms_period],
        cert_chainable_on: vec![],
    };
    // Write Identity
    identities_db.write(|db| {
        db.insert(idty.idty_doc.issuers()[0], idty.clone());
    })?;
    // Write membership
    ms_db.write(|db| {
        let mut memberships = db.get(&ms_created_block_id).cloned().unwrap_or_default();
        memberships.insert(wot_id);
        db.insert(ms_created_block_id, memberships);
    })?;
    Ok(())
}

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

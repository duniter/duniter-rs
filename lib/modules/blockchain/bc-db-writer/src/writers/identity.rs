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

use crate::{BinFreeStructDb, Db, DbError, MsExpirV10Datas};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockNumber, Blockstamp};
use dubp_currency_params::CurrencyParameters;
use dubp_user_docs::documents::identity::IdentityDocumentV10;
use dup_crypto::keys::PubKey;
use dup_crypto::keys::PublicKey;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::indexes::identities::{DbIdentity, DbIdentityState};
use durs_bc_db_reader::{DbReadable, DbValue};
use durs_common_tools::fatal_error;
use durs_wot::WotId;

/// Remove identity from databases
pub fn revert_create_identity(
    db: &Db,
    ms_db: &BinFreeStructDb<MsExpirV10Datas>,
    pubkey: &PubKey,
) -> Result<(), DbError> {
    let dal_idty = durs_bc_db_reader::indexes::identities::get_identity(db, pubkey)?
        .expect("Try to revert unexist idty.");
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
    db.write(|mut w| {
        db.get_store(IDENTITIES).delete(
            w.as_mut(),
            &dal_idty.idty_doc.issuers()[0].to_bytes_vector(),
        )?;
        Ok(w)
    })?;
    Ok(())
}

/// Write identity in databases
pub fn create_identity(
    currency_params: &CurrencyParameters,
    db: &Db,
    ms_db: &BinFreeStructDb<MsExpirV10Datas>,
    idty_doc: &IdentityDocumentV10,
    ms_created_block_id: BlockNumber,
    wot_id: WotId,
    current_blockstamp: Blockstamp,
    current_bc_time: u64,
) -> Result<(), DbError> {
    let mut idty_doc = idty_doc.clone();
    idty_doc.reduce();
    let idty = DbIdentity {
        hash: "0".to_string(),
        state: DbIdentityState::Member(vec![0]),
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
    let bin_idty = durs_dbs_tools::to_bytes(&idty)?;
    db.write(|mut w| {
        db.get_store(IDENTITIES).put(
            w.as_mut(),
            &idty.idty_doc.issuers()[0].to_bytes_vector(),
            &DbValue::Blob(&bin_idty),
        )?;
        Ok(w)
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
    db: &Db,
    pubkey: &PubKey,
    exclusion_blockstamp: &Blockstamp,
    revert: bool,
) -> Result<(), DbError> {
    let mut idty_datas = durs_bc_db_reader::indexes::identities::get_identity(db, pubkey)?
        .expect("Try to exclude unexist idty.");
    idty_datas.state = if revert {
        match idty_datas.state {
            DbIdentityState::ExpireMember(renewed_counts) => {
                DbIdentityState::Member(renewed_counts)
            }
            _ => fatal_error!("Try to revert exclusion for a no excluded identity !"),
        }
    } else {
        match idty_datas.state {
            DbIdentityState::Member(renewed_counts) => {
                DbIdentityState::ExpireMember(renewed_counts)
            }
            _ => fatal_error!("Try to exclude for an already excluded/revoked identity !"),
        }
    };
    idty_datas.expired_on = if revert {
        None
    } else {
        Some(*exclusion_blockstamp)
    };
    // Write new identity datas
    let bin_idty = durs_dbs_tools::to_bytes(&idty_datas)?;
    db.write(|mut w| {
        db.get_store(IDENTITIES).put(
            w.as_mut(),
            &pubkey.to_bytes_vector(),
            &DbValue::Blob(&bin_idty),
        )?;
        Ok(w)
    })?;
    Ok(())
}

/// Apply "revoke identity" event
pub fn revoke_identity(
    db: &Db,
    pubkey: &PubKey,
    renewal_blockstamp: &Blockstamp,
    explicit: bool,
    revert: bool,
) -> Result<(), DbError> {
    let mut member_datas = durs_bc_db_reader::indexes::identities::get_identity(db, pubkey)?
        .expect("Try to revoke unexist idty.");

    member_datas.state = if revert {
        match member_datas.state {
            DbIdentityState::ExplicitRevoked(renewed_counts) => {
                DbIdentityState::Member(renewed_counts)
            }
            DbIdentityState::ExplicitExpireRevoked(renewed_counts)
            | DbIdentityState::ImplicitRevoked(renewed_counts) => {
                DbIdentityState::ExpireMember(renewed_counts)
            }
            _ => fatal_error!("Try to revert revoke_identity() for a no revoked idty !"),
        }
    } else {
        match member_datas.state {
            DbIdentityState::ExpireMember(renewed_counts) => {
                DbIdentityState::ExplicitExpireRevoked(renewed_counts)
            }
            DbIdentityState::Member(renewed_counts) => {
                if explicit {
                    DbIdentityState::ExplicitRevoked(renewed_counts)
                } else {
                    DbIdentityState::ImplicitRevoked(renewed_counts)
                }
            }
            _ => fatal_error!("Try to revert revoke an already revoked idty !"),
        }
    };
    member_datas.revoked_on = if revert {
        None
    } else {
        Some(*renewal_blockstamp)
    };

    // Update idty
    let bin_idty = durs_dbs_tools::to_bytes(&member_datas)?;
    db.write(|mut w| {
        db.get_store(IDENTITIES).put(
            w.as_mut(),
            &pubkey.to_bytes_vector(),
            &DbValue::Blob(&bin_idty),
        )?;
        Ok(w)
    })?;
    Ok(())
}

/// Apply "renewal identity" event in databases
pub fn renewal_identity(
    currency_params: &CurrencyParameters,
    db: &Db,
    ms_db: &BinFreeStructDb<MsExpirV10Datas>,
    pubkey: &PubKey,
    idty_wot_id: WotId,
    renewal_timestamp: u64,
    ms_created_block_id: BlockNumber,
    revert: bool,
) -> Result<(), DbError> {
    // Get idty_datas
    let mut idty_datas = durs_bc_db_reader::indexes::identities::get_identity(db, pubkey)?
        .expect("Fatal error : try to renewal unknow identity !");
    // Calculate new state value
    idty_datas.state = if revert {
        match idty_datas.state {
            DbIdentityState::Member(renewed_counts) => {
                let mut new_renewed_counts = renewed_counts.clone();
                new_renewed_counts[renewed_counts.len() - 1] -= 1;
                if new_renewed_counts[renewed_counts.len() - 1] > 0 {
                    DbIdentityState::Member(new_renewed_counts)
                } else {
                    DbIdentityState::ExpireMember(new_renewed_counts)
                }
            }
            _ => fatal_error!("Try to revert renewal_identity() for an excluded or revoked idty !"),
        }
    } else {
        match idty_datas.state {
            DbIdentityState::Member(renewed_counts) => {
                let mut new_renewed_counts = renewed_counts.clone();
                new_renewed_counts[renewed_counts.len() - 1] += 1;
                DbIdentityState::Member(new_renewed_counts)
            }
            DbIdentityState::ExpireMember(renewed_counts) => {
                let mut new_renewed_counts = renewed_counts.clone();
                new_renewed_counts.push(0);
                DbIdentityState::Member(new_renewed_counts)
            }
            _ => fatal_error!("Try to renewed a revoked identity !"),
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
    let bin_idty = durs_dbs_tools::to_bytes(&idty_datas)?;
    db.write(|mut w| {
        db.get_store(IDENTITIES).put(
            w.as_mut(),
            &pubkey.to_bytes_vector(),
            &DbValue::Blob(&bin_idty),
        )?;
        Ok(w)
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
pub fn remove_identity(db: &Db, pubkey: PubKey) -> Result<(), DbError> {
    db.write(|mut w| {
        db.get_store(IDENTITIES)
            .delete(w.as_mut(), &pubkey.to_bytes_vector())?;
        Ok(w)
    })
}

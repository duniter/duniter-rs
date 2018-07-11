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
use duniter_crypto::keys::PubKey;
use duniter_documents::blockchain::v10::documents::IdentityDocument;
use duniter_documents::blockchain::Document;
use duniter_documents::{BlockId, Blockstamp};
use duniter_wotb::NodeId;
use identity::{DALIdentity, DALIdentityState};
use {BinDB, DALError, IdentitiesV10Datas, MsExpirV10Datas};

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

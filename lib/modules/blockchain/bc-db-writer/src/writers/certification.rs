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

use crate::{BinFreeStructDb, Db, DbError};
use dubp_common_doc::BlockNumber;
use dubp_currency_params::CurrencyParameters;
use dubp_user_docs::documents::certification::CompactCertificationDocumentV10;
use dup_crypto::keys::*;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::indexes::identities::DbIdentity;
use durs_bc_db_reader::{CertsExpirV10Datas, DbReadable, DbValue};
use durs_wot::WotId;

/// Apply "certification" event in databases
pub fn write_certification(
    currency_params: &CurrencyParameters,
    db: &Db,
    certs_db: &BinFreeStructDb<CertsExpirV10Datas>,
    source_pubkey: PubKey,
    source: WotId,
    target: WotId,
    created_block_id: BlockNumber,
    written_timestamp: u64,
) -> Result<(), DbError> {
    // Get cert_chainable_on
    let mut member_datas =
        durs_bc_db_reader::indexes::identities::get_identity(db, &source_pubkey)?
            .expect("Try to write certification with unexist certifier.");
    // Push new cert_chainable_on
    member_datas
        .cert_chainable_on
        .push(written_timestamp + currency_params.sig_period);
    // Write new identity datas
    let bin_member_datas = durs_dbs_tools::to_bytes(&member_datas)?;
    db.write(|mut w| {
        db.get_store(IDENTITIES).put(
            w.as_mut(),
            &source_pubkey.to_bytes_vector(),
            &DbValue::Blob(&bin_member_datas),
        )?;
        Ok(w)
    })?;
    // Add cert in certs_db
    certs_db.write(|db| {
        let mut created_certs = db.get(&created_block_id).cloned().unwrap_or_default();
        created_certs.insert((source, target));
        db.insert(created_block_id, created_certs);
    })?;
    Ok(())
}

/// Revert writtent certification
pub fn revert_write_cert(
    db: &Db,
    certs_db: &BinFreeStructDb<CertsExpirV10Datas>,
    compact_doc: CompactCertificationDocumentV10,
    source: WotId,
    target: WotId,
) -> Result<(), DbError> {
    // Remove CertsExpirV10Datas entry
    certs_db.write(|db| {
        let mut certs = db
            .get(&compact_doc.block_number)
            .cloned()
            .unwrap_or_default();
        certs.remove(&(source, target));
        db.insert(compact_doc.block_number, certs);
    })?;
    // Pop last cert_chainable_on
    db.write(|mut w| {
        let identities_store = db.get_store(IDENTITIES);
        let pubkey_bytes = compact_doc.issuer.to_bytes_vector();
        if let Some(v) = identities_store.get(w.as_ref(), &pubkey_bytes)? {
            let mut member_datas = Db::from_db_value::<DbIdentity>(v)?;
            member_datas.cert_chainable_on.pop();
            let bin_member_datas = durs_dbs_tools::to_bytes(&member_datas)?;
            identities_store.put(w.as_mut(), &pubkey_bytes, &DbValue::Blob(&bin_member_datas))?
        }
        Ok(w)
    })?;
    Ok(())
}

/// Revert "certification expiry" event in databases
pub fn revert_expire_cert(
    certs_db: &BinFreeStructDb<CertsExpirV10Datas>,
    source: WotId,
    target: WotId,
    created_block_id: BlockNumber,
) -> Result<(), DbError> {
    // Reinsert CertsExpirV10Datas entry
    certs_db.write(|db| {
        let mut certs = db.get(&created_block_id).cloned().unwrap_or_default();
        certs.insert((source, target));
        db.insert(created_block_id, certs);
    })?;
    Ok(())
}

/// Apply "certification expiry" event in databases
pub fn expire_certs(
    certs_db: &BinFreeStructDb<CertsExpirV10Datas>,
    created_block_id: BlockNumber,
) -> Result<(), DbError> {
    // Remove CertsExpirV10Datas entries
    certs_db.write(|db| {
        db.remove(&created_block_id);
    })?;
    Ok(())
}

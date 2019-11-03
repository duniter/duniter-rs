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

//! Certifications stored indexes: write requests.

use crate::{Db, DbError, DbWriter};
use dubp_common_doc::BlockNumber;
use dubp_currency_params::CurrencyParameters;
use dubp_user_docs::documents::certification::CompactCertificationDocumentV10;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::indexes::identities::DbIdentity;
use durs_bc_db_reader::{DbReadable, DbValue};
use durs_wot::WotId;

/// Apply "certification" event in databases
pub fn write_certification(
    currency_params: &CurrencyParameters,
    db: &Db,
    w: &mut DbWriter,
    source: WotId,
    target: WotId,
    created_block_id: BlockNumber,
    written_timestamp: u64,
) -> Result<(), DbError> {
    // Get cert_chainable_on
    let mut member_datas =
        durs_bc_db_reader::indexes::identities::get_identity_by_wot_id(db, w.as_ref(), source)?
            .expect("Try to write certification with unexist certifier.");
    // Push new cert_chainable_on
    member_datas
        .cert_chainable_on
        .push(written_timestamp + currency_params.sig_period);
    // Write new identity datas
    let bin_member_datas = durs_dbs_tools::to_bytes(&member_datas)?;
    db.get_int_store(IDENTITIES).put(
        w.as_mut(),
        source.0 as u32,
        &DbValue::Blob(&bin_member_datas),
    )?;
    // Add cert in certs_db
    db.get_multi_int_store(CERTS_BY_CREATED_BLOCK).put(
        w.as_mut(),
        created_block_id.0,
        &DbValue::U64(cert_to_u64(source, target)),
    )?;
    Ok(())
}

/// Revert writtent certification
pub fn revert_write_cert(
    db: &Db,
    w: &mut DbWriter,
    compact_doc: CompactCertificationDocumentV10,
    source: WotId,
    target: WotId,
) -> Result<(), DbError> {
    // Remove CertsExpirV10Datas entry
    db.get_multi_int_store(CERTS_BY_CREATED_BLOCK).delete(
        w.as_mut(),
        compact_doc.block_number.0,
        &DbValue::U64(cert_to_u64(source, target)),
    )?;

    // Pop last cert_chainable_on
    let identities_store = db.get_int_store(IDENTITIES);
    if let Some(v) = identities_store.get(w.as_ref(), source.0 as u32)? {
        let mut member_datas = Db::from_db_value::<DbIdentity>(v)?;
        member_datas.cert_chainable_on.pop();
        let bin_member_datas = durs_dbs_tools::to_bytes(&member_datas)?;
        identities_store.put(
            w.as_mut(),
            source.0 as u32,
            &DbValue::Blob(&bin_member_datas),
        )?
    }
    Ok(())
}

/// Revert "certification expiry" event in databases
pub fn revert_expire_cert(
    db: &Db,
    w: &mut DbWriter,
    source: WotId,
    target: WotId,
    created_block_id: BlockNumber,
) -> Result<(), DbError> {
    // Reinsert CertsExpirV10Datas entry
    db.get_multi_int_store(CERTS_BY_CREATED_BLOCK).put(
        w.as_mut(),
        created_block_id.0,
        &DbValue::U64(cert_to_u64(source, target)),
    )?;
    Ok(())
}

/// Apply "certification expiry" event in databases
pub fn expire_certs(
    db: &Db,
    w: &mut DbWriter,
    created_block_id: BlockNumber,
) -> Result<(), DbError> {
    // Remove all certs created at block `created_block_id`
    if db
        .get_multi_int_store(CERTS_BY_CREATED_BLOCK)
        .get_first(w.as_ref(), created_block_id.0)?
        .is_some()
    {
        db.get_multi_int_store(CERTS_BY_CREATED_BLOCK)
            .delete_all(w.as_mut(), created_block_id.0)?;
    }

    Ok(())
}

#[inline]
fn cert_to_u64(source: WotId, target: WotId) -> u64 {
    durs_common_tools::fns::_u64::from_2_u32(source.0 as u32, target.0 as u32)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cert_to_u64() {
        assert_eq!(1u64, cert_to_u64(WotId(0), WotId(1)));
        assert_eq!(2_623u64, cert_to_u64(WotId(0), WotId(2_623)));
        assert_eq!((std::u32::MAX as u64) + 1, cert_to_u64(WotId(1), WotId(0)));
    }
}

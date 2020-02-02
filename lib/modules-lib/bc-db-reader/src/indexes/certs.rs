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

//! Certificatiosn stored index.

use crate::*;
use dubp_common_doc::BlockNumber;
use durs_dbs_tools::DbError;
use durs_wot::WotId;
use std::collections::HashMap;

/// Find certifications that emitted in indicated blocks expiring
pub fn find_expire_certs<DB: BcDbInReadTx>(
    db: &DB,
    blocks_expiring: &[BlockNumber],
) -> Result<HashMap<(WotId, WotId), BlockNumber>, DbError> {
    let mut all_expire_certs = HashMap::new();
    for expire_block_id in blocks_expiring {
        for entry_result in db
            .db()
            .get_multi_int_store(CERTS_BY_CREATED_BLOCK)
            .get(db.r(), expire_block_id.0)?
        {
            if let Some(value) = entry_result?.1 {
                if let DbValue::U64(cert) = value {
                    let (source, target) = cert_from_u64(cert);
                    all_expire_certs.insert((source, target), *expire_block_id);
                } else {
                    return Err(DbError::DBCorrupted);
                }
            }
        }
    }
    Ok(all_expire_certs)
}

#[inline]
fn cert_from_u64(cert: u64) -> (WotId, WotId) {
    let (source, target) = durs_common_tools::fns::_u64::to_2_u32(cert);

    (WotId(source as usize), WotId(target as usize))
}

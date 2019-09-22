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

//! Universal dividends stored indexes: write requests.

use crate::*;
use dubp_common_doc::BlockNumber;
use dup_crypto::keys::PubKey;
use durs_bc_db_reader::constants::DIVIDENDS;
use durs_bc_db_reader::indexes::sources::SourceAmount;
use durs_bc_db_reader::DbValue;

/// Apply UD creation in databases
pub fn create_du(
    db: &Db,
    w: &mut DbWriter,
    du_amount: &SourceAmount,
    du_block_id: BlockNumber,
    members: &[PubKey],
    revert: bool,
) -> Result<(), DbError> {
    debug!(
        "create_du(amount, block_id, members, revert)=({:?}, {}, {:?}, {})",
        du_amount, du_block_id.0, members, revert
    );
    // Insert/Remove UD sources in UDsV10DB
    for pubkey in members {
        let pubkey_bytes = pubkey.to_bytes_vector();
        if revert {
            db.get_multi_store(DIVIDENDS).delete(
                w.as_mut(),
                &pubkey_bytes,
                &DbValue::U64(u64::from(du_block_id.0)),
            )?;
        } else {
            db.get_multi_store(DIVIDENDS).put(
                w.as_mut(),
                &pubkey_bytes,
                &DbValue::U64(u64::from(du_block_id.0)),
            )?;
        }
    }
    Ok(())
}

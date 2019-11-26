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

//! Curent meta datas storage: define write requests.

use crate::*;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_block_doc::BlockDocument;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::constants::CURRENT_METADATA;
use durs_bc_db_reader::current_metadata::CurrentMetaDataKey;
use durs_bc_db_reader::DbValue;

/// Update CURRENT_METADATA
pub fn update_current_metadata(
    db: &Db,
    w: &mut DbWriter,
    new_current_block: &BlockDocument,
) -> Result<(), DbError> {
    let new_current_blockstamp_bytes: Vec<u8> = new_current_block.blockstamp().into();

    // Update current blockstamp
    db.get_int_store(CURRENT_METADATA).put(
        w.as_mut(),
        CurrentMetaDataKey::CurrentBlockstamp.to_u32(),
        &DbValue::Blob(&new_current_blockstamp_bytes),
    )?;
    // Update current common time (also named "blockchain time")
    db.get_int_store(CURRENT_METADATA).put(
        w.as_mut(),
        CurrentMetaDataKey::CurrentBlockchainTime.to_u32(),
        &DbValue::U64(new_current_block.common_time()),
    )?;

    Ok(())
}

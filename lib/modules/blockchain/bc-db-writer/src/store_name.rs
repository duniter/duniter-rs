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

//! "store name" storage: define write requests.

use crate::{Db, DbWriter};
use dubp_block_doc::BlockDocument;
use durs_dbs_tools::DbError;

pub(crate) fn update_store_name(
    _db: &Db,
    _w: &mut DbWriter,
    _new_current_block: &BlockDocument,
) -> Result<(), DbError> {
    //unimplemented!()
    Ok(())
}

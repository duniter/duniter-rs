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

use crate::*;
use dubp_common_doc::Blockstamp;

/// get current blockstamp
pub fn get_current_blockstamp(forks_dbs: &ForksDBs) -> Result<Option<Blockstamp>, DALError> {
    Ok(forks_dbs
        .fork_tree_db
        .read(|fork_tree| fork_tree.get_current_blockstamp())?)
}

/// Get stackables blocks
pub fn get_stackables_blocks(
    forks_dbs: &ForksDBs,
    current_blockstamp: &Blockstamp,
) -> Result<Vec<DALBlock>, DALError> {
    if let Some(stackables_blocks) = forks_dbs
        .orphan_blocks_db
        .read(|db| db.get(&current_blockstamp).cloned())?
    {
        Ok(stackables_blocks)
    } else {
        Ok(vec![])
    }
}

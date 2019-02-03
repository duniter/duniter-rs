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

use crate::constants::MAX_FORKS;
use crate::*;
use dubp_documents::{BlockId, Blockstamp, PreviousBlockstamp};

/// Get fork block
pub fn get_block_fork(
    forks_db: &BinDB<ForksV10Datas>,
    previous_blockstamp: &PreviousBlockstamp,
) -> Result<Option<ForkId>, DALError> {
    Ok(forks_db.read(|forks_db| {
        for (fork_id, fork_meta_datas) in forks_db {
            if fork_meta_datas.contains_key(&previous_blockstamp) {
                return Some(*fork_id);
            }
        }
        None
    })?)
}

/// Get stackables blocks
pub fn get_stackables_blocks(
    forks_db: &BinDB<ForksV10Datas>,
    forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
    current_blockstamp: &Blockstamp,
) -> Result<Vec<DALBlock>, DALError> {
    debug!("get_stackables_blocks() after {}", current_blockstamp);
    let stackables_blocks_hashs = forks_db.read(|db| {
        let mut stackables_blocks_hashs = Vec::new();
        for fork_meta_datas in db.values() {
            if let Some(block_hash) = fork_meta_datas.get(&current_blockstamp) {
                stackables_blocks_hashs.push(*block_hash);
            }
        }
        stackables_blocks_hashs
    })?;
    let stackables_blocks = forks_blocks_db.read(|db| {
        let mut stackables_blocks = Vec::new();
        for stackable_block_hash in stackables_blocks_hashs {
            if let Some(dal_block) = db.get(&Blockstamp {
                id: BlockId(current_blockstamp.id.0 + 1),
                hash: stackable_block_hash,
            }) {
                stackables_blocks.push(dal_block.clone());
            }
        }
        stackables_blocks
    })?;
    Ok(stackables_blocks)
}
/// Get stackables forks
pub fn get_stackables_forks(
    db: &BinDB<ForksV10Datas>,
    current_blockstamp: &Blockstamp,
) -> Result<Vec<usize>, DALError> {
    Ok(db.read(|db| {
        let mut stackables_forks = Vec::new();
        for f in 0..*MAX_FORKS {
            if let Some(fork_meta_datas) = db.get(&ForkId(f)) {
                if fork_meta_datas.get(&current_blockstamp).is_some() {
                    stackables_forks.push(f);
                }
            }
        }
        stackables_forks
    })?)
}

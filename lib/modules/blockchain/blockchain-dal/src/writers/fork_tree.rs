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
use dubp_documents::*;

/// Insert new head Block in fork tree
pub fn insert_new_head_block(
    fork_tree_db: &BinDB<ForksTreeV10Datas>,
    blockstamp: Blockstamp,
) -> Result<(), DALError> {
    fork_tree_db.write(|fork_tree| {
        let parent_id_opt = fork_tree.get_main_branch_node_id(BlockId(blockstamp.id.0 - 1));
        fork_tree.insert_new_node(blockstamp, parent_id_opt, true);
    })?;

    Ok(())
}

/// Insert new fork block in fork tree only if parent exist in fork tree (orphan block not inserted)
/// Returns true if block has a parent and has therefore been inserted, return false if block is orphaned
pub fn insert_new_fork_block(
    fork_tree_db: &BinDB<ForksTreeV10Datas>,
    blockstamp: PreviousBlockstamp,
) -> Result<bool, DALError> {
    let parent_id_opt =
        fork_tree_db.read(|fork_tree| fork_tree.find_node_with_blockstamp(&blockstamp))?;

    if let Some(parent_id) = parent_id_opt {
        fork_tree_db.write(|fork_tree| {
            fork_tree.insert_new_node(blockstamp, Some(parent_id), false);
        })?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/*************************************
 * BEGIN OLD FORK SYSTEM (TO REMOVE)
 *************************************/

/// Insert fork Block in databases
/// return NodeId of block in tree, or None if block not inserted
pub fn insert_fork_block(
    _fork_tree_db: &BinDB<ForksTreeV10Datas>,
    _fork_blocks: &BinDB<ForksBlocksV10Datas>,
    _orphan_blocks: &BinDB<OrphanBlocksV10Datas>,
    _dal_block: &DALBlock,
) -> Result<Option<id_tree::NodeId>, DALError> {
    // TODO
    unimplemented!()
}

/// Delete fork
pub fn delete_fork(
    forks_db: &BinDB<ForksV10Datas>,
    forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
    fork_id: ForkId,
) -> Result<(), DALError> {
    let fork_meta_datas = forks_db
        .read(|forks_db| forks_db.get(&fork_id).cloned())?
        .expect("Fatal error : try to delete unknow fork");
    // Remove fork blocks
    forks_blocks_db.write(|db| {
        for (previous_blockstamp, hash) in fork_meta_datas {
            let blockstamp = Blockstamp {
                id: BlockId(previous_blockstamp.id.0 + 1),
                hash,
            };
            db.remove(&blockstamp);
        }
    })?;
    // Remove fork meta datas
    forks_db.write_safe(|db| {
        db.remove(&fork_id);
    })?;
    Ok(())
}
/// Assign fork id to new block
pub fn assign_fork_to_new_block(
    forks_db: &BinDB<ForksV10Datas>,
    new_block_previous_blockstamp: &PreviousBlockstamp,
    new_block_hash: &BlockHash,
) -> Result<(Option<ForkId>, bool), DALError> {
    let forks_meta_datas = forks_db.read(|forks_db| forks_db.clone())?;
    // Try to assign block to an existing fork
    for (fork_id, fork_meta_datas) in &forks_meta_datas {
        let mut fork_datas = fork_meta_datas.clone();
        for (previous_blockstamp, hash) in fork_meta_datas {
            let blockstamp = Blockstamp {
                id: BlockId(previous_blockstamp.id.0 + 1),
                hash: *hash,
            };
            if *new_block_previous_blockstamp == blockstamp {
                fork_datas.insert(*new_block_previous_blockstamp, *new_block_hash);
                forks_db.write(|forks_db| {
                    forks_db.insert(*fork_id, fork_datas);
                })?;
                return Ok((Some(*fork_id), false));
            }
        }
    }
    // Find an available fork
    let mut new_fork_id = ForkId(0);
    for f in 0..*MAX_FORKS {
        if !forks_meta_datas.contains_key(&ForkId(f)) {
            new_fork_id = ForkId(f);
            break;
        }
    }
    if new_fork_id.0 == 0 {
        if forks_meta_datas.len() >= *MAX_FORKS {
            return Ok((None, false));
        } else {
            new_fork_id = ForkId(forks_meta_datas.len());
        }
    }
    // Create new fork
    let mut new_fork = HashMap::new();
    new_fork.insert(*new_block_previous_blockstamp, *new_block_hash);
    forks_db.write(|forks_db| {
        forks_db.insert(new_fork_id, new_fork);
    })?;
    Ok((Some(new_fork_id), true))
}

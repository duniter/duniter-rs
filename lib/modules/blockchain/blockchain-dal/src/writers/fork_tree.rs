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

use crate::entities::fork_tree::ForkTree;
use crate::*;
use dubp_documents::*;

/// Insert new head Block in fork tree,
/// return vector of removed blockstamps
pub fn insert_new_head_block(
    fork_tree_db: &BinDB<ForksTreeV10Datas>,
    blockstamp: Blockstamp,
) -> Result<Vec<Blockstamp>, DALError> {
    fork_tree_db.write(|fork_tree| {
        let parent_id_opt = if blockstamp.id.0 > 0 && fork_tree.size() > 0 {
            Some(fork_tree.get_main_branch_node_id(BlockNumber(blockstamp.id.0 - 1))
                .expect("Fatal error: fail to insert new head block : previous block not exist in main branch"))
        } else {
            None
        };
        fork_tree.insert_new_node(blockstamp, parent_id_opt, true);
    })?;

    Ok(fork_tree_db.read(ForkTree::get_removed_blockstamps)?)
}

/// Insert new fork block in fork tree only if parent exist in fork tree (orphan block not inserted)
/// Returns true if block has a parent and has therefore been inserted, return false if block is orphaned
pub fn insert_new_fork_block(
    fork_tree_db: &BinDB<ForksTreeV10Datas>,
    blockstamp: Blockstamp,
    previous_hash: Hash,
) -> Result<bool, DALError> {
    let previous_blockstamp = Blockstamp {
        id: BlockNumber(blockstamp.id.0 - 1),
        hash: BlockHash(previous_hash),
    };

    let parent_id_opt =
        fork_tree_db.read(|fork_tree| fork_tree.find_node_with_blockstamp(&previous_blockstamp))?;

    if let Some(parent_id) = parent_id_opt {
        fork_tree_db.write(|fork_tree| {
            fork_tree.insert_new_node(blockstamp, Some(parent_id), false);
        })?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Modify the main branch (function to call after a successful roolback)
pub fn change_main_branch(
    forks_dbs: &ForksDBs,
    old_current_blockstamp: Blockstamp,
    new_current_blockstamp: Blockstamp,
) -> Result<(), DALError> {
    forks_dbs.fork_tree_db.write(|tree| {
        tree.change_main_branch(old_current_blockstamp, new_current_blockstamp);
    })?;

    let removed_blockstamps = forks_dbs
        .fork_tree_db
        .read(ForkTree::get_removed_blockstamps)?;

    // Remove too old blocks
    forks_dbs.fork_blocks_db.write(|db| {
        for blockstamp in removed_blockstamps {
            db.remove(&blockstamp);
        }
    })?;

    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::entities::fork_tree::TreeNodeId;

    #[test]
    fn test_insert_new_head_block() -> Result<(), DALError> {
        // Create mock datas
        let blockstamps = dubp_documents_tests_tools::mocks::generate_blockstamps(
            *crate::constants::FORK_WINDOW_SIZE + 2,
        );
        let fork_tree_db = open_db::<ForksTreeV10Datas>(None, "")?;

        // Insert genesis block
        assert_eq!(
            Ok(vec![]),
            insert_new_head_block(&fork_tree_db, blockstamps[0])
        );

        // Check tree state
        assert_eq!(1, fork_tree_db.read(|tree| tree.size())?);
        assert_eq!(
            vec![(TreeNodeId(0), blockstamps[0])],
            fork_tree_db.read(|tree| tree.get_sheets())?
        );

        // Insert FORK_WINDOW_SIZE blocks
        for i in 1..*crate::constants::FORK_WINDOW_SIZE {
            assert_eq!(
                Ok(vec![]),
                insert_new_head_block(&fork_tree_db, blockstamps[i])
            );
        }

        // Check tree state
        assert_eq!(
            *crate::constants::FORK_WINDOW_SIZE,
            fork_tree_db.read(|tree| tree.size())?
        );
        assert_eq!(
            vec![(
                TreeNodeId(*crate::constants::FORK_WINDOW_SIZE - 1),
                blockstamps[*crate::constants::FORK_WINDOW_SIZE - 1]
            )],
            fork_tree_db.read(|tree| tree.get_sheets())?
        );

        // Insert blocks after FORK_WINDOW_SIZE (firsts blocks must be removed)
        assert_eq!(
            Ok(vec![blockstamps[0]]),
            insert_new_head_block(
                &fork_tree_db,
                blockstamps[*crate::constants::FORK_WINDOW_SIZE]
            )
        );
        assert_eq!(
            Ok(vec![blockstamps[1]]),
            insert_new_head_block(
                &fork_tree_db,
                blockstamps[*crate::constants::FORK_WINDOW_SIZE + 1]
            )
        );

        Ok(())
    }

    #[test]
    fn test_insert_new_fork_block() -> Result<(), DALError> {
        // Create mock datas
        let blockstamps = dubp_documents_tests_tools::mocks::generate_blockstamps(
            *crate::constants::FORK_WINDOW_SIZE + 3,
        );
        let fork_tree_db = open_db::<ForksTreeV10Datas>(None, "")?;

        // Insert 4 main blocks
        for i in 0..4 {
            assert_eq!(
                Ok(vec![]),
                insert_new_head_block(&fork_tree_db, blockstamps[i])
            );
        }

        // Check tree state
        assert_eq!(4, fork_tree_db.read(|tree| tree.size())?);
        assert_eq!(
            vec![(TreeNodeId(3), blockstamps[3])],
            fork_tree_db.read(|tree| tree.get_sheets())?
        );

        // Insert first fork block at child of block 2
        let fork_blockstamp = Blockstamp {
            id: BlockNumber(3),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
        };
        assert_eq!(
            Ok(true),
            insert_new_fork_block(&fork_tree_db, fork_blockstamp, blockstamps[2].hash.0)
        );

        // Check tree state
        assert_eq!(5, fork_tree_db.read(|tree| tree.size())?);
        assert!(rust_tests_tools::collections::slice_same_elems(
            &vec![
                (TreeNodeId(3), blockstamps[3]),
                (TreeNodeId(4), fork_blockstamp)
            ],
            &fork_tree_db.read(|tree| tree.get_sheets())?
        ));

        // Insert second fork block at child of first fork block
        let fork_blockstamp_2 = Blockstamp {
            id: BlockNumber(4),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('B')),
        };
        assert_eq!(
            Ok(true),
            insert_new_fork_block(&fork_tree_db, fork_blockstamp_2, fork_blockstamp.hash.0)
        );

        // Check tree state
        assert_eq!(6, fork_tree_db.read(|tree| tree.size())?);
        assert!(rust_tests_tools::collections::slice_same_elems(
            &vec![
                (TreeNodeId(3), blockstamps[3]),
                (TreeNodeId(5), fork_blockstamp_2)
            ],
            &fork_tree_db.read(|tree| tree.get_sheets())?
        ));

        // Insert FORK_WINDOW_SIZE blocks
        for i in 4..*crate::constants::FORK_WINDOW_SIZE {
            assert_eq!(
                Ok(vec![]),
                insert_new_head_block(&fork_tree_db, blockstamps[i])
            );
        }

        // Check tree state
        assert_eq!(
            *crate::constants::FORK_WINDOW_SIZE + 2,
            fork_tree_db.read(|tree| tree.size())?
        );
        assert!(rust_tests_tools::collections::slice_same_elems(
            &vec![
                (
                    TreeNodeId(*crate::constants::FORK_WINDOW_SIZE + 1),
                    blockstamps[*crate::constants::FORK_WINDOW_SIZE - 1]
                ),
                (TreeNodeId(5), fork_blockstamp_2)
            ],
            &fork_tree_db.read(|tree| tree.get_sheets())?
        ));

        // Insert 2 new main blocks (too old blocks must be removed)
        for i in 0..2 {
            assert_eq!(
                Ok(vec![blockstamps[i]]),
                insert_new_head_block(
                    &fork_tree_db,
                    blockstamps[*crate::constants::FORK_WINDOW_SIZE + i]
                )
            );
        }

        // Insert one new main block (fork branch must be removed)
        assert_eq!(
            Ok(vec![blockstamps[2], fork_blockstamp_2, fork_blockstamp]),
            insert_new_head_block(
                &fork_tree_db,
                blockstamps[*crate::constants::FORK_WINDOW_SIZE + 2]
            )
        );

        // Check tree state
        assert_eq!(
            *crate::constants::FORK_WINDOW_SIZE,
            fork_tree_db.read(|tree| tree.size())?
        );
        assert_eq!(
            vec![(
                TreeNodeId(*crate::constants::FORK_WINDOW_SIZE + 4),
                blockstamps[*crate::constants::FORK_WINDOW_SIZE + 2]
            )],
            fork_tree_db.read(|tree| tree.get_sheets())?
        );

        Ok(())
    }
}

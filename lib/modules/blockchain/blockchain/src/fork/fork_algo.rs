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

use dubp_common_doc::Blockstamp;
use dubp_documents::documents::block::BlockDocumentTrait;
use durs_blockchain_dal::entities::fork_tree::ForkTree;
use durs_blockchain_dal::{DALError, ForksDBs};
use std::collections::HashSet;

/// Number of advance blocks required
pub static ADVANCE_BLOCKS: &'static u32 = &3;
/// Advance blockchain time required (in seconds)
pub static ADVANCE_TIME: &'static u64 = &900;

pub fn fork_resolution_algo(
    forks_dbs: &ForksDBs,
    fork_window_size: usize,
    current_blockstamp: Blockstamp,
    invalid_blocks: &HashSet<Blockstamp>,
) -> Result<Option<Vec<Blockstamp>>, DALError> {
    let current_bc_time = forks_dbs.fork_blocks_db.read(|db| {
        db.get(&current_blockstamp)
            .expect("safe unwrap")
            .block
            .common_time()
    })?;

    debug!(
        "fork_resolution_algo({}, {})",
        fork_window_size, current_bc_time
    );

    let mut sheets = forks_dbs.fork_tree_db.read(ForkTree::get_sheets)?;

    sheets.sort_unstable_by(|s1, s2| s2.1.id.cmp(&s1.1.id));

    for sheet in sheets {
        if sheet.1 != current_blockstamp {
            let branch = forks_dbs
                .fork_tree_db
                .read(|fork_tree| fork_tree.get_fork_branch(sheet.0))?;

            if branch.is_empty() {
                continue;
            }

            let branch_head_blockstamp = branch.last().expect("safe unwrap");
            let branch_head_median_time = forks_dbs.fork_blocks_db.read(|db| {
                db.get(&branch_head_blockstamp)
                    .expect("safe unwrap")
                    .block
                    .common_time()
            })?;

            if branch_head_blockstamp.id.0 >= current_blockstamp.id.0 + *ADVANCE_BLOCKS
                && branch_head_median_time >= current_bc_time + *ADVANCE_TIME
                && branch[0].id.0 + fork_window_size as u32 > current_blockstamp.id.0
            {
                debug!(
                    "fork_resolution_algo() found eligible fork branch #{}:",
                    branch_head_blockstamp
                );
                let mut valid_branch = true;
                for blockstamp in &branch {
                    if invalid_blocks.contains(blockstamp) {
                        valid_branch = false;
                        break;
                    }
                }

                if valid_branch {
                    debug!(
                        "fork_resolution_algo() found valid fork branch #{}:",
                        branch_head_blockstamp
                    );
                    return Ok(Some(branch));
                }
            }
        }
    }

    debug!("fork_resolution_algo() return Ok(None)");
    Ok(None)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::*;
    use dubp_common_doc::{BlockHash, BlockNumber};
    use dubp_documents::documents::block::BlockDocument;
    use durs_blockchain_dal::entities::block::DALBlock;

    #[test]
    fn test_fork_resolution_algo() -> Result<(), DALError> {
        // Get FORK_WINDOW_SIZE value
        let fork_window_size = *dubp_currency_params::constants::DEFAULT_FORK_WINDOW_SIZE;

        // Open empty databases in memory mode
        let bc_dbs = BlocksV10DBs::open(None);
        let forks_dbs = ForksDBs::open(None);

        // Begin with no invalid blocks
        let invalid_blocks: HashSet<Blockstamp> = HashSet::new();

        // Generate `FORK_WINDOW_SIZE + 2` mock blocks
        let main_branch: Vec<BlockDocument> =
            dubp_documents_tests_tools::mocks::gen_empty_timed_blocks_v10(
                fork_window_size + 2,
                0u64,
            );

        // Insert mock blocks in forks_dbs
        for block in &main_branch {
            durs_blockchain_dal::writers::block::insert_new_head_block(
                &bc_dbs.blockchain_db,
                &forks_dbs,
                DALBlock {
                    block: block.clone(),
                    expire_certs: None,
                },
            )?;
        }
        assert_eq!(
            fork_window_size,
            forks_dbs.fork_tree_db.read(|fork_tree| fork_tree.size())?
        );
        assert_eq!(
            fork_window_size,
            forks_dbs.fork_blocks_db.read(|db| db.len())?
        );

        // Get current blockstamp
        let mut current_blockstamp = forks_dbs
            .fork_tree_db
            .read(|fork_tree| fork_tree.get_sheets())?
            .get(0)
            .expect("must be one sheet")
            .1;

        // Generate 3 fork block
        let fork_point = &main_branch[main_branch.len() - 2];
        let fork_blocks: Vec<BlockDocument> = (0..3)
            .map(|i| {
                BlockDocument::V10(
                    dubp_documents_tests_tools::mocks::gen_empty_timed_block_v10(
                        Blockstamp {
                            id: BlockNumber(fork_point.number().0 + i + 1),
                            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
                        },
                        ADVANCE_TIME - 1,
                        if i == 0 {
                            fork_point.hash().expect("safe unwrap").0
                        } else {
                            dup_crypto_tests_tools::mocks::hash('A')
                        },
                    ),
                )
            })
            .collect();

        // Add forks blocks into fork tree
        insert_fork_blocks(&forks_dbs, &fork_blocks)?;
        assert_eq!(
            2,
            forks_dbs
                .fork_tree_db
                .read(|tree| tree.get_sheets().len())?
        );

        // Must not fork
        assert_eq!(
            None,
            fork_resolution_algo(
                &forks_dbs,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            )?
        );

        // Add the determining fork block
        let determining_blockstamp = Blockstamp {
            id: BlockNumber(fork_point.number().0 + 4),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
        };
        assert_eq!(
            true,
            durs_blockchain_dal::writers::block::insert_new_fork_block(
                &forks_dbs,
                DALBlock {
                    block: BlockDocument::V10(
                        dubp_documents_tests_tools::mocks::gen_empty_timed_block_v10(
                            determining_blockstamp,
                            *ADVANCE_TIME,
                            dup_crypto_tests_tools::mocks::hash('A'),
                        )
                    ),
                    expire_certs: None,
                },
            )?,
        );

        // Must fork
        assert_eq!(
            Some(vec![
                fork_blocks[0].blockstamp(),
                fork_blocks[1].blockstamp(),
                fork_blocks[2].blockstamp(),
                determining_blockstamp,
            ]),
            fork_resolution_algo(
                &forks_dbs,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            )?
        );
        current_blockstamp = determining_blockstamp;

        // The old main branch catches up and overlaps with the fork
        let new_main_blocks: Vec<BlockDocument> = (0..7)
            .map(|i| {
                BlockDocument::V10(
                    dubp_documents_tests_tools::mocks::gen_empty_timed_block_v10(
                        Blockstamp {
                            id: BlockNumber(fork_point.number().0 + i + 1),
                            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('B')),
                        },
                        ADVANCE_TIME * 2,
                        if i == 0 {
                            fork_point.hash().expect("safe unwrap").0
                        } else {
                            dup_crypto_tests_tools::mocks::hash('B')
                        },
                    ),
                )
            })
            .collect();
        insert_fork_blocks(&forks_dbs, &new_main_blocks)?;

        // Must refork
        assert_eq!(
            Some(new_main_blocks.iter().map(|b| b.blockstamp()).collect()),
            fork_resolution_algo(
                &forks_dbs,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            )?
        );
        //current_blockstamp = new_main_blocks.last().expect("safe unwrap").blockstamp();

        Ok(())
    }

    fn insert_fork_blocks(forks_dbs: &ForksDBs, blocks: &[BlockDocument]) -> Result<(), DALError> {
        for block in blocks {
            assert_eq!(
                true,
                durs_blockchain_dal::writers::block::insert_new_fork_block(
                    forks_dbs,
                    DALBlock {
                        block: block.clone(),
                        expire_certs: None,
                    },
                )?,
            );
        }

        Ok(())
    }
}

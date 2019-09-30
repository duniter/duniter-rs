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

use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::Blockstamp;
use durs_bc_db_reader::blocks::fork_tree::ForkTree;
use durs_bc_db_reader::{DbReadable, DbReader};
use durs_bc_db_writer::DbError;
use std::collections::HashSet;

/// Number of advance blocks required
pub static ADVANCE_BLOCKS: &u32 = &3;
/// Advance blockchain time required (in seconds)
pub static ADVANCE_TIME: &u64 = &900;

pub fn fork_resolution_algo<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    fork_tree: &ForkTree,
    fork_window_size: usize,
    current_blockstamp: Blockstamp,
    invalid_blocks: &HashSet<Blockstamp>,
) -> Result<Option<Vec<Blockstamp>>, DbError> {
    let current_bc_time = durs_bc_db_reader::current_meta_datas::get_current_common_time_(db, r)?;

    debug!(
        "fork_resolution_algo({}, {})",
        fork_window_size, current_bc_time
    );

    let mut sheets = fork_tree.get_sheets();

    sheets.sort_unstable_by(|s1, s2| s2.1.id.cmp(&s1.1.id));

    for sheet in sheets {
        if sheet.1 != current_blockstamp {
            let branch = fork_tree.get_fork_branch(sheet.0);

            if branch.is_empty() {
                continue;
            }

            let branch_head_blockstamp = branch.last().expect("safe unwrap");
            let branch_head_median_time =
                durs_bc_db_reader::blocks::get_fork_block(db, r, *branch_head_blockstamp)?
                    .unwrap_or_else(|| {
                        panic!(
                        "Db corrupted: fork block {} referenced in fork tree but not exist in db.",
                        branch_head_blockstamp
                    )
                    })
                    .block
                    .common_time();

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
    use dubp_block_doc::BlockDocument;
    use dubp_common_doc::{BlockHash, BlockNumber};
    use durs_bc_db_reader::blocks::DbBlock;

    #[test]
    fn test_fork_resolution_algo() -> Result<(), DbError> {
        // Open empty DB in tmp dir
        let db = crate::tests::open_tmp_db()?;
        let mut fork_tree = ForkTree::default();

        // Get FORK_WINDOW_SIZE value
        let fork_window_size = *dubp_currency_params::constants::DEFAULT_FORK_WINDOW_SIZE;

        // Begin with no invalid blocks
        let invalid_blocks: HashSet<Blockstamp> = HashSet::new();

        // Generate `FORK_WINDOW_SIZE + 2` mock blocks
        let main_branch: Vec<BlockDocument> =
            dubp_blocks_tests_tools::mocks::gen_empty_timed_blocks_v10(fork_window_size + 2, 0u64);

        // Insert mock blocks in forks_dbs
        db.write(|mut w| {
            for block in &main_branch {
                durs_bc_db_writer::current_meta_datas::update_current_meta_datas(
                    &db, &mut w, &block,
                )?;
                durs_bc_db_writer::blocks::insert_new_head_block(
                    &db,
                    &mut w,
                    Some(&mut fork_tree),
                    DbBlock {
                        block: block.clone(),
                        expire_certs: None,
                    },
                )?;
            }
            Ok(w)
        })?;

        // Local blockchain must contain at least `fork_window_size +2` blocks
        assert!(db
            .read(
                |r| durs_bc_db_reader::blocks::get_block_in_local_blockchain(
                    &db,
                    r,
                    BlockNumber((fork_window_size + 1) as u32)
                )
            )?
            .is_some());

        // Fork tree must contain at least `fork_window_size +2` blocks
        assert_eq!(fork_window_size, fork_tree.size());

        // Get current blockstamp
        let mut current_blockstamp = fork_tree.get_sheets().get(0).expect("must be one sheet").1;

        // Generate 3 fork block
        let fork_point = &main_branch[main_branch.len() - 2];
        let fork_blocks: Vec<BlockDocument> = (0..3)
            .map(|i| {
                BlockDocument::V10(dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10(
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
                ))
            })
            .collect();

        // Add forks blocks into fork tree
        insert_fork_blocks(&db, &mut fork_tree, &fork_blocks)?;
        assert_eq!(2, fork_tree.get_sheets().len());

        // Must not fork
        assert_eq!(
            None,
            db.read(|r| fork_resolution_algo(
                &db,
                r,
                &fork_tree,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            ))?
        );

        // Add the determining fork block
        let determining_blockstamp = Blockstamp {
            id: BlockNumber(fork_point.number().0 + 4),
            hash: BlockHash(dup_crypto_tests_tools::mocks::hash('A')),
        };
        db.write(|mut w| {
            assert_eq!(
                true,
                durs_bc_db_writer::blocks::insert_new_fork_block(
                    &db,
                    &mut w,
                    &mut fork_tree,
                    DbBlock {
                        block: BlockDocument::V10(
                            dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10(
                                determining_blockstamp,
                                *ADVANCE_TIME,
                                dup_crypto_tests_tools::mocks::hash('A'),
                            )
                        ),
                        expire_certs: None,
                    },
                )?,
            );
            Ok(w)
        })?;

        // Must fork
        assert_eq!(
            Some(vec![
                fork_blocks[0].blockstamp(),
                fork_blocks[1].blockstamp(),
                fork_blocks[2].blockstamp(),
                determining_blockstamp,
            ]),
            db.read(|r| fork_resolution_algo(
                &db,
                r,
                &mut fork_tree,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            ))?
        );
        current_blockstamp = determining_blockstamp;

        // The old main branch catches up and overlaps with the fork
        let new_main_blocks: Vec<BlockDocument> = (0..7)
            .map(|i| {
                BlockDocument::V10(dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10(
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
                ))
            })
            .collect();
        insert_fork_blocks(&db, &mut fork_tree, &new_main_blocks)?;

        // Must refork
        assert_eq!(
            Some(new_main_blocks.iter().map(|b| b.blockstamp()).collect()),
            db.read(|r| fork_resolution_algo(
                &db,
                r,
                &mut fork_tree,
                fork_window_size,
                current_blockstamp,
                &invalid_blocks
            ))?
        );
        //current_blockstamp = new_main_blocks.last().expect("safe unwrap").blockstamp();

        Ok(())
    }

    fn insert_fork_blocks(
        db: &Db,
        fork_tree: &mut ForkTree,
        blocks: &[BlockDocument],
    ) -> Result<(), DbError> {
        db.write(|mut w| {
            for block in blocks {
                assert_eq!(
                    true,
                    durs_bc_db_writer::blocks::insert_new_fork_block(
                        db,
                        &mut w,
                        fork_tree,
                        DbBlock {
                            block: block.clone(),
                            expire_certs: None,
                        },
                    )?,
                );
            }
            Ok(w)
        })
    }
}

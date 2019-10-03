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

use dubp_block_doc::BlockDocument;
use dubp_blocks_tests_tools::mocks::gen_empty_block_v10_with_issuer_and_pow_min as gen_empty_block;
use dubp_common_doc::BlockNumber;
use dup_crypto_tests_tools::mocks::pubkey;
use durs_bc_db_reader::current_frame::*;
use durs_bc_db_reader::DbReadable;
use durs_bc_db_writer::blocks::insert_new_head_block;
use durs_bc_db_writer::DbError;
use durs_wot::WotId;

mod common;

const INITIAL_POW_MIN: usize = 70;

//#[ignore]
#[test]
fn test_current_frame() -> Result<(), DbError> {
    // Open temporary database
    let db = common::open_tmp_db()?;

    // Create and insert fake wot index
    let issuer_a = pubkey('A');
    db.write(|mut w| {
        common::insert_wot_index_entry(&db, &mut w, WotId(0), issuer_a)?;
        Ok(w)
    })?;

    // Insert genesis block
    let genesis_block =
        BlockDocument::V10(gen_empty_block(BlockNumber(0), issuer_a, INITIAL_POW_MIN));
    db.write(|mut w| {
        insert_new_head_block(&db, &mut w, None, common::to_db_block(genesis_block))?;
        Ok(w)
    })?;

    // Verify current frame #0
    let current_frame = get_current_frame(&db)?;
    assert_eq!(1, current_frame.len());
    assert_eq!(
        (
            WotId(0),
            MemberInCurrentFrame {
                forged_blocks: 1,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 0,
                },
            }
        ),
        current_frame[0]
    );

    // Insert block #1
    let block_1 = BlockDocument::V10(gen_empty_block(
        BlockNumber(1),
        issuer_a,
        INITIAL_POW_MIN + 1,
    ));
    db.write(|mut w| {
        insert_new_head_block(&db, &mut w, None, common::to_db_block(block_1))?;
        Ok(w)
    })?;

    // Verify current frame #1
    let current_frame = get_current_frame(&db)?;
    assert_eq!(2, current_frame.len());
    assert_eq!(
        (
            WotId(0),
            MemberInCurrentFrame {
                forged_blocks: 2,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 0,
                },
            }
        ),
        current_frame[0]
    );
    assert_eq!(
        PersonalDifficulty {
            exclusion_factor: 1,
            handicap: 2,
        },
        db.read(|r| get_member_diffi(&db, r, WotId(0)))?
    );

    // Insert block #2
    let issuer_b = pubkey('B');
    let block_2 = BlockDocument::V10(gen_empty_block(
        BlockNumber(2),
        issuer_b,
        INITIAL_POW_MIN + 2,
    ));
    db.write(|mut w| {
        insert_new_head_block(&db, &mut w, None, common::to_db_block(block_2))?;
        Ok(w)
    })?;
    // Verify current frame #2
    let current_frame = get_current_frame(&db)?;
    assert_eq!(3, current_frame.len());
    assert_eq!(
        (
            WotId(0),
            MemberInCurrentFrame {
                forged_blocks: 2,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 2,
                },
            }
        ),
        current_frame[0]
    );
    assert_eq!(
        (
            WotId(1),
            MemberInCurrentFrame {
                forged_blocks: 1,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 2,
                },
            }
        ),
        current_frame[1]
    );

    // Insert block #3
    let block_3 = BlockDocument::V10(gen_empty_block(
        BlockNumber(3),
        issuer_b,
        INITIAL_POW_MIN + 3,
    ));
    db.write(|mut w| {
        insert_new_head_block(&db, &mut w, None, common::to_db_block(block_3))?;
        Ok(w)
    })?;
    // Verify current frame #3
    let current_frame = get_current_frame(&db)?;
    assert_eq!(4, current_frame.len());
    assert_eq!(
        (
            WotId(0),
            MemberInCurrentFrame {
                forged_blocks: 2,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 2,
                },
            }
        ),
        current_frame[0]
    );
    assert_eq!(
        (
            WotId(1),
            MemberInCurrentFrame {
                forged_blocks: 2,
                difficulty: PersonalDifficulty {
                    exclusion_factor: 1,
                    handicap: 2,
                },
            }
        ),
        current_frame[1]
    );

    Ok(())
}

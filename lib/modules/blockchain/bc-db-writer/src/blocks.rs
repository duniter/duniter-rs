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

//! Blocks storage: defien write requests.

pub mod fork_tree;

use crate::*;
use dubp_block_doc::block::BlockDocumentTrait;
use dubp_common_doc::traits::Document;
use durs_bc_db_reader::blocks::fork_tree::ForkTree;
use durs_bc_db_reader::blocks::DbBlock;
use durs_bc_db_reader::constants::*;
use durs_bc_db_reader::DbValue;
use unwrap::unwrap;

/// Insert new head Block in databases
pub fn insert_new_head_block(
    db: &Db,
    w: &mut DbWriter,
    fork_tree: Option<&mut ForkTree>,
    dal_block: DbBlock,
) -> Result<(), DbError> {
    // Serialize datas
    let bin_dal_block = durs_dbs_tools::to_bytes(&dal_block)?;

    let main_blocks_store = db.get_int_store(MAIN_BLOCKS);
    let fork_blocks_store = db.get_store(FORK_BLOCKS);

    // Insert block in MAIN_BLOCKS store
    main_blocks_store.put(
        w.as_mut(),
        *dal_block.block.number(),
        &Db::db_value(&bin_dal_block)?,
    )?;

    // Update current meta datas
    crate::current_meta_datas::update_current_meta_datas(db, w, &dal_block.block)?;

    // Update stores linked to MAIN_BLOCKS
    crate::store_name::update_store_name(db, w, &dal_block.block)?;

    if let Some(fork_tree) = fork_tree {
        // Insert head block in fork tree
        let removed_blockstamps =
            crate::blocks::fork_tree::insert_new_head_block(fork_tree, dal_block.blockstamp())?;
        // Insert head block in ForkBlocks
        let blockstamp_bytes: Vec<u8> = dal_block.blockstamp().into();
        fork_blocks_store.put(
            w.as_mut(),
            &blockstamp_bytes,
            &Db::db_value(&bin_dal_block)?,
        )?;
        // Remove too old blocks
        for blockstamp in removed_blockstamps {
            let blockstamp_bytes: Vec<u8> = blockstamp.into();
            if fork_blocks_store
                .get(w.as_ref(), &blockstamp_bytes)?
                .is_some()
            {
                fork_blocks_store.delete(w.as_mut(), &blockstamp_bytes)?;
            }
        }
    }
    Ok(())
}

/// Remove a block in local blockchain storage
pub fn remove_block(db: &Db, w: &mut DbWriter, block_number: BlockNumber) -> Result<(), DbError> {
    db.get_int_store(MAIN_BLOCKS)
        .delete(w.as_mut(), block_number.0)?;
    Ok(())
}

/// Insert new fork Block in databases
pub fn insert_new_fork_block(
    db: &Db,
    w: &mut DbWriter,
    fork_tree: &mut ForkTree,
    dal_block: DbBlock,
) -> Result<bool, DbError> {
    let bin_dal_block = durs_dbs_tools::to_bytes(&dal_block)?;
    let blockstamp_bytes: Vec<u8> = dal_block.blockstamp().into();
    if fork_tree::insert_new_fork_block(
        fork_tree,
        dal_block.block.blockstamp(),
        unwrap!(dal_block.block.previous_hash()),
    )? {
        // Insert fork block FORK_BLOCKS
        db.get_store(FORK_BLOCKS).put(
            w.as_mut(),
            &blockstamp_bytes,
            &Db::db_value(&bin_dal_block)?,
        )?;

        // As long as orphan blocks can succeed the last inserted block, they are inserted
        for stackable_block in
            durs_bc_db_reader::blocks::get_stackables_blocks(db, dal_block.blockstamp())?
        {
            let _ = insert_new_fork_block(db, w, fork_tree, stackable_block);
        }

        Ok(true)
    } else {
        // Insert block in OrphanBlocks store
        let previous_blockstamp_bytes: Vec<u8> = dal_block.previous_blockstamp().into();
        let orphan_blockstamps_store = db.get_store(ORPHAN_BLOCKSTAMP);
        let mut orphan_blockstamps = if let Some(v) =
            orphan_blockstamps_store.get(w.as_ref(), &previous_blockstamp_bytes)?
        {
            Db::from_db_value::<Vec<Blockstamp>>(v)?
        } else {
            vec![]
        };
        orphan_blockstamps.push(dal_block.blockstamp());
        orphan_blockstamps_store.put(
            w.as_mut(),
            &previous_blockstamp_bytes,
            &DbValue::Blob(&durs_dbs_tools::to_bytes(&orphan_blockstamps)?),
        )?;
        // Insert orphan block in FORK_BLOCKS
        db.get_store(FORK_BLOCKS).put(
            w.as_mut(),
            &blockstamp_bytes,
            &Db::db_value(&bin_dal_block)?,
        )?;
        Ok(false)
    }
}

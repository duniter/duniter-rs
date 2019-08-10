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

use crate::entities::block::DALBlock;
use crate::*;
use crate::{BinDB, DALError, LocalBlockchainV10Datas};
use dubp_documents::documents::block::BlockDocumentTrait;
use dubp_documents::Document;
use unwrap::unwrap;

/// Insert new head Block in databases
pub fn insert_new_head_block(
    blockchain_db: &BinDB<LocalBlockchainV10Datas>,
    forks_dbs: &ForksDBs,
    dal_block: DALBlock,
) -> Result<(), DALError> {
    // Insert head block in blockchain
    blockchain_db.write(|db| {
        db.insert(dal_block.block.number(), dal_block.clone());
    })?;

    // Insert head block in fork tree
    let removed_blockstamps = crate::writers::fork_tree::insert_new_head_block(
        &forks_dbs.fork_tree_db,
        dal_block.blockstamp(),
    )?;

    // Insert head block in ForksBlocks
    forks_dbs.fork_blocks_db.write(|db| {
        db.insert(dal_block.blockstamp(), dal_block);
    })?;

    // Remove too old blocks
    forks_dbs.fork_blocks_db.write(|db| {
        for blockstamp in removed_blockstamps {
            db.remove(&blockstamp);
        }
    })?;

    Ok(())
}

/// Insert new fork Block in databases
pub fn insert_new_fork_block(forks_dbs: &ForksDBs, dal_block: DALBlock) -> Result<bool, DALError> {
    if crate::writers::fork_tree::insert_new_fork_block(
        &forks_dbs.fork_tree_db,
        dal_block.block.blockstamp(),
        unwrap!(dal_block.block.previous_hash()),
    )? {
        // Insert in ForksBlocks
        forks_dbs.fork_blocks_db.write(|db| {
            db.insert(dal_block.blockstamp(), dal_block.clone());
        })?;

        // As long as orphan blocks can succeed the last inserted block, they are inserted
        if let Some(stackables_blocks) = forks_dbs
            .orphan_blocks_db
            .read(|db| db.get(&dal_block.blockstamp()).cloned())?
        {
            for stackable_block in stackables_blocks {
                let _ = insert_new_fork_block(forks_dbs, stackable_block);
            }
        }

        Ok(true)
    } else {
        let previous_blockstamp = dal_block.previous_blockstamp();

        // Get orphanBlocks vector
        let mut orphan_blocks = if let Some(orphan_blocks) = forks_dbs
            .orphan_blocks_db
            .read(|db| db.get(&previous_blockstamp).cloned())?
        {
            orphan_blocks
        } else {
            Vec::new()
        };

        // Add fork block
        orphan_blocks.push(dal_block);

        // Update OrphanBlocks DB
        forks_dbs.orphan_blocks_db.write(|db| {
            db.insert(previous_blockstamp, orphan_blocks);
        })?;

        Ok(false)
    }
}

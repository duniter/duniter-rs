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

use crate::constants::*;
use crate::*;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
use dup_crypto::keys::*;
use std::collections::HashMap;
use unwrap::unwrap;

/// Get block hash
pub fn get_block_hash<DB: DbReadable>(
    db: &DB,
    block_number: BlockNumber,
) -> Result<Option<BlockHash>, DALError> {
    Ok(
        if let Some(block) = get_block_in_local_blockchain(db, block_number)? {
            block.hash()
        } else {
            None
        },
    )
}
/// Return true if the node already knows this block
pub fn already_have_block<DB: DbReadable>(
    db: &DB,
    forks_dbs: &ForksDBs,
    blockstamp: Blockstamp,
    previous_hash: Option<Hash>,
) -> Result<bool, DALError> {
    let previous_blockstamp = if blockstamp.id.0 > 0 {
        Blockstamp {
            id: BlockNumber(blockstamp.id.0 - 1),
            hash: BlockHash(unwrap!(previous_hash)),
        }
    } else {
        Blockstamp::default()
    };

    if forks_dbs
        .fork_blocks_db
        .read(|db| db.contains_key(&blockstamp))?
    {
        return Ok(true);
    } else if let Some(orphan_blockstamps) = forks_dbs.orphan_blocks_db.read(|db| {
        if let Some(orphan_blocks) = db.get(&previous_blockstamp) {
            let orphan_blockstamps: Vec<Blockstamp> =
                orphan_blocks.iter().map(DALBlock::blockstamp).collect();
            Some(orphan_blockstamps)
        } else {
            None
        }
    })? {
        for orphan_blockstamp in orphan_blockstamps {
            if orphan_blockstamp == blockstamp {
                return Ok(true);
            }
        }
    } else {
        return Ok(get_block_in_local_blockchain(db, blockstamp.id)?.is_some());
    }

    Ok(false)
}

/// Get block
pub fn get_block<DB: DbReadable>(
    db: &DB,
    forks_blocks_db: Option<&BinFreeStructDb<ForksBlocksV10Datas>>,
    blockstamp: &Blockstamp,
) -> Result<Option<DALBlock>, DALError> {
    let opt_dal_block = get_dal_block_in_local_blockchain(db, blockstamp.id)?;
    if opt_dal_block.is_none() && forks_blocks_db.is_some() {
        Ok(forks_blocks_db
            .expect("safe unwrap")
            .read(|db| db.get(&blockstamp).cloned())?)
    } else {
        Ok(opt_dal_block)
    }
}

/// Get block in local blockchain
#[inline]
pub fn get_block_in_local_blockchain<DB: DbReadable>(
    db: &DB,
    block_number: BlockNumber,
) -> Result<Option<BlockDocument>, DALError> {
    Ok(get_dal_block_in_local_blockchain(db, block_number)?.map(|dal_block| dal_block.block))
}

/// Get block in local blockchain
pub fn get_dal_block_in_local_blockchain<DB: DbReadable>(
    db: &DB,
    block_number: BlockNumber,
) -> Result<Option<DALBlock>, DALError> {
    db.read(|r| {
        if let Some(v) = db.get_int_store(LOCAL_BC).get(r, block_number.0)? {
            Ok(Some(DB::from_db_value(v)?))
        } else {
            Ok(None)
        }
    })
    //local_bc_db.read(|r| local_bc_db.get(&r, block_number.0))
}

/// Get several blocks in local blockchain
pub fn get_blocks_in_local_blockchain<DB: DbReadable>(
    db: &DB,
    first_block_number: BlockNumber,
    mut count: u32,
) -> Result<Vec<BlockDocument>, DALError> {
    db.read(|r| {
        let bc_store = db.get_int_store(LOCAL_BC);
        let mut blocks = Vec::with_capacity(count as usize);
        let mut current_block_number = first_block_number;

        while let Some(v) = bc_store.get(r, current_block_number.0)? {
            blocks.push(DB::from_db_value::<DALBlock>(v)?.block);
            count -= 1;
            if count > 0 {
                current_block_number = BlockNumber(current_block_number.0 + 1);
            } else {
                return Ok(blocks);
            }
        }
        Ok(blocks)
    })
    /*bc_db.read(|r| {
        let mut blocks = Vec::with_capacity(count as usize);
        let mut current_block_number = first_block_number;
        while let Some(dal_block) = bc_db.get(&r, current_block_number.0)? {
            blocks.push(dal_block.block);
            count -= 1;
            if count > 0 {
                current_block_number = BlockNumber(current_block_number.0 + 1);
            } else {
                return Ok(blocks);
            }
        }
        Ok(blocks)
    })*/
}

/// Get current frame of calculating members
pub fn get_current_frame<DB: DbReadable>(
    current_block: &DALBlock,
    db: &DB,
) -> Result<HashMap<PubKey, usize>, DALError> {
    let frame_begin =
        current_block.block.number().0 - current_block.block.current_frame_size() as u32;

    let blocks = get_blocks_in_local_blockchain(
        db,
        BlockNumber(frame_begin),
        current_block.block.current_frame_size() as u32,
    )?;

    let mut current_frame: HashMap<PubKey, usize> = HashMap::new();
    for block in blocks {
        let issuer = block.issuers()[0];
        let issuer_count_blocks = if let Some(issuer_count_blocks) = current_frame.get(&issuer) {
            issuer_count_blocks + 1
        } else {
            1
        };
        current_frame.insert(issuer, issuer_count_blocks);
    }

    Ok(current_frame)
}

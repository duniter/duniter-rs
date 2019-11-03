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

//! Define blocks entities and requests

pub mod fork_tree;

use crate::constants::*;
use crate::*;
use dubp_block_doc::block::{BlockDocument, BlockDocumentTrait};
use dubp_common_doc::traits::Document;
use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp, PreviousBlockstamp};
use dup_crypto::hashs::Hash;
use dup_crypto::keys::*;
use durs_dbs_tools::DbError;
use durs_wot::WotId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A block as it is saved in a database
pub struct DbBlock {
    /// Block document
    pub block: BlockDocument,
    /// List of certifications that expire in this block.
    /// Warning : BlockNumber contain the emission block, not the written block !
    /// HashMap<(Source, Target), BlockNumber>
    pub expire_certs: Option<HashMap<(WotId, WotId), BlockNumber>>,
}

impl DbBlock {
    /// Get blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        self.block.blockstamp()
    }
    /// Get previous blockstamp
    pub fn previous_blockstamp(&self) -> PreviousBlockstamp {
        self.block.previous_blockstamp()
    }
}

/// Return true if the node already knows this block
pub fn already_have_block<DB: DbReadable>(
    db: &DB,
    blockstamp: Blockstamp,
    previous_hash: Option<Hash>,
) -> Result<bool, DbError> {
    db.read(|r| {
        let blockstamp_bytes: Vec<u8> = blockstamp.into();
        if db
            .get_store(FORK_BLOCKS)
            .get(r, &blockstamp_bytes)?
            .is_some()
        {
            return Ok(true);
        } else if blockstamp.id > BlockNumber(0) {
            let previous_blockstamp_bytes: Vec<u8> = PreviousBlockstamp {
                id: BlockNumber(blockstamp.id.0 - 1),
                hash: BlockHash(previous_hash.expect("no genesis block must have previous hash")),
            }
            .into();
            if let Some(v) = db
                .get_store(ORPHAN_BLOCKSTAMP)
                .get(r, &previous_blockstamp_bytes)?
            {
                for orphan_blockstamp in DB::from_db_value::<Vec<Blockstamp>>(v)? {
                    if orphan_blockstamp == blockstamp {
                        return Ok(true);
                    }
                }
            }
        }
        if let Some(v) = db.get_int_store(MAIN_BLOCKS).get(r, blockstamp.id.0)? {
            if DB::from_db_value::<DbBlock>(v)?.block.blockstamp() == blockstamp {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    })
}

/// Get block
pub fn get_block<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    blockstamp: Blockstamp,
) -> Result<Option<DbBlock>, DbError> {
    let opt_dal_block = get_dal_block_in_local_blockchain(db, r, blockstamp.id)?;
    if opt_dal_block.is_none() {
        get_fork_block(db, r, blockstamp)
    } else {
        Ok(opt_dal_block)
    }
}

/// Get fork block
pub fn get_fork_block<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    blockstamp: Blockstamp,
) -> Result<Option<DbBlock>, DbError> {
    let blockstamp_bytes: Vec<u8> = blockstamp.into();
    if let Some(v) = db.get_store(FORK_BLOCKS).get(r, &blockstamp_bytes)? {
        Ok(Some(DB::from_db_value(v)?))
    } else {
        Ok(None)
    }
}

/// Get block hash
pub fn get_block_hash<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    block_number: BlockNumber,
) -> Result<Option<BlockHash>, DbError> {
    Ok(
        if let Some(block) = get_block_in_local_blockchain(db, r, block_number)? {
            block.hash()
        } else {
            None
        },
    )
}

/// Get block in local blockchain
#[inline]
pub fn get_block_in_local_blockchain<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    block_number: BlockNumber,
) -> Result<Option<BlockDocument>, DbError> {
    Ok(get_db_block_in_local_blockchain(db, r, block_number)?.map(|dal_block| dal_block.block))
}

/// Get block in local blockchain
pub fn get_db_block_in_local_blockchain<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    block_number: BlockNumber,
) -> Result<Option<DbBlock>, DbError> {
    if let Some(v) = db.get_int_store(MAIN_BLOCKS).get(r, block_number.0)? {
        Ok(Some(DB::from_db_value(v)?))
    } else {
        Ok(None)
    }
}

/// Get several blocks in local blockchain
pub fn get_blocks_in_local_blockchain<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    first_block_number: BlockNumber,
    mut count: u32,
) -> Result<Vec<BlockDocument>, DbError> {
    let bc_store = db.get_int_store(MAIN_BLOCKS);
    let mut blocks = Vec::with_capacity(count as usize);
    let mut current_block_number = first_block_number;

    while let Some(v) = bc_store.get(r, current_block_number.0)? {
        blocks.push(DB::from_db_value::<DbBlock>(v)?.block);
        count -= 1;
        if count > 0 {
            current_block_number = BlockNumber(current_block_number.0 + 1);
        } else {
            return Ok(blocks);
        }
    }
    Ok(blocks)
}

/// Get several blocks in local blockchain by their number
pub fn get_blocks_in_local_blockchain_by_numbers<DB: DbReadable, R: DbReader>(
    db: &DB,
    r: &R,
    numbers: Vec<BlockNumber>,
) -> Result<Vec<DbBlock>, DbError> {
    numbers
        .into_iter()
        .filter_map(|n| match get_db_block_in_local_blockchain(db, r, n) {
            Ok(Some(db_block)) => Some(Ok(db_block)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<Vec<DbBlock>, DbError>>()
}

/// Get current frame of calculating members
pub fn get_current_frame<DB: DbReadable>(
    current_block: &BlockDocument,
    db: &DB,
) -> Result<HashMap<PubKey, usize>, DbError> {
    let frame_begin = current_block.number().0 - current_block.current_frame_size() as u32;

    let blocks = db.read(|r| {
        get_blocks_in_local_blockchain(
            db,
            r,
            BlockNumber(frame_begin),
            current_block.current_frame_size() as u32,
        )
    })?;

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

/// Get stackables blocks
#[inline]
pub fn get_stackables_blocks<DB: DbReadable>(
    db: &DB,
    current_blockstamp: Blockstamp,
) -> Result<Vec<DbBlock>, DbError> {
    get_orphan_blocks(db, current_blockstamp)
}

/// Get orphan blocks
pub fn get_orphan_blocks<DB: DbReadable>(
    db: &DB,
    blockstamp: PreviousBlockstamp,
) -> Result<Vec<DbBlock>, DbError> {
    let blockstamp_bytes: Vec<u8> = blockstamp.into();
    db.read(|r| {
        if let Some(v) = db.get_store(ORPHAN_BLOCKSTAMP).get(r, &blockstamp_bytes)? {
            let orphan_blockstamps = DB::from_db_value::<Vec<Blockstamp>>(v)?;
            let mut orphan_blocks = Vec::with_capacity(orphan_blockstamps.len());
            for orphan_blockstamp in orphan_blockstamps {
                let orphan_blockstamp_bytes: Vec<u8> = orphan_blockstamp.into();
                if let Some(v) = db.get_store(FORK_BLOCKS).get(r, &orphan_blockstamp_bytes)? {
                    orphan_blocks.push(DB::from_db_value::<DbBlock>(v)?);
                } else {
                    return Err(DbError::DBCorrupted);
                }
            }
            Ok(orphan_blocks)
        } else {
            Ok(vec![])
        }
    })
}

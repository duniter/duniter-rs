//  Copyright (C) 2018  The Durs Project Developers.
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

use crate::*;
use dubp_documents::documents::block::{BlockDocument, BlockDocumentTrait};
use dubp_documents::Document;
use dubp_documents::{BlockHash, BlockNumber, Blockstamp};
use dup_crypto::keys::*;
use std::collections::HashMap;
use unwrap::unwrap;

/// get current blockstamp
pub fn get_current_blockstamp(blocks_db: &BlocksV10DBs) -> Result<Option<Blockstamp>, DALError> {
    Ok(blocks_db.blockchain_db.read(|db| {
        let blockchain_len = db.len() as u32;
        if blockchain_len == 0 {
            None
        } else if let Some(dal_block) = db.get(&BlockNumber(blockchain_len - 1)) {
            Some(dal_block.blockstamp())
        } else {
            None
        }
    })?)
}

/// Get block hash
pub fn get_block_hash(
    db: &BinDB<LocalBlockchainV10Datas>,
    block_number: BlockNumber,
) -> Result<Option<BlockHash>, DALError> {
    Ok(db.read(|db| {
        if let Some(dal_block) = db.get(&block_number) {
            dal_block.block.hash()
        } else {
            None
        }
    })?)
}
/// Return true if the node already knows this block
pub fn already_have_block(
    blockchain_db: &BinDB<LocalBlockchainV10Datas>,
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
        return Ok(blockchain_db.read(|db| {
            if let Some(dal_block) = db.get(&blockstamp.id) {
                if dal_block.block.hash().unwrap_or_default() == blockstamp.hash {
                    return true;
                }
            }
            false
        })?);
    }

    Ok(false)
}

/// Get block
pub fn get_block(
    blockchain_db: &BinDB<LocalBlockchainV10Datas>,
    forks_blocks_db: Option<&BinDB<ForksBlocksV10Datas>>,
    blockstamp: &Blockstamp,
) -> Result<Option<DALBlock>, DALError> {
    let dal_block = blockchain_db.read(|db| db.get(&blockstamp.id).cloned())?;
    if dal_block.is_none() && forks_blocks_db.is_some() {
        Ok(forks_blocks_db
            .expect("safe unwrap")
            .read(|db| db.get(&blockstamp).cloned())?)
    } else {
        Ok(dal_block)
    }
}

/// Get block in local blockchain
#[inline]
pub fn get_block_in_local_blockchain(
    db: &BinDB<LocalBlockchainV10Datas>,
    block_id: BlockNumber,
) -> Result<Option<BlockDocument>, DALError> {
    Ok(db.read(|db| {
        if let Some(dal_block) = db.get(&block_id) {
            Some(dal_block.block.clone())
        } else {
            None
        }
    })?)
}

/// Get several blocks in local blockchain
#[inline]
pub fn get_blocks_in_local_blockchain(
    db: &BinDB<LocalBlockchainV10Datas>,
    first_block_number: BlockNumber,
    count: u32,
) -> Result<Vec<BlockDocument>, DALError> {
    Ok(db.read(|db| {
        let mut blocks = Vec::with_capacity(count as usize);
        let mut current_block_number = first_block_number;
        while let Some(dal_block) = db.get(&current_block_number) {
            blocks.push(dal_block.block.clone());
            current_block_number = BlockNumber(current_block_number.0 + 1);
        }
        blocks
    })?)
}

/// Get current frame of calculating members
pub fn get_current_frame(
    current_block: &DALBlock,
    db: &BinDB<LocalBlockchainV10Datas>,
) -> Result<HashMap<PubKey, usize>, DALError> {
    let frame_begin =
        current_block.block.number().0 - current_block.block.current_frame_size() as u32;
    Ok(db.read(|db| {
        let mut current_frame: HashMap<PubKey, usize> = HashMap::new();
        for block_number in frame_begin..current_block.block.number().0 {
            let issuer = db
                .get(&BlockNumber(block_number))
                .unwrap_or_else(|| fatal_error!("Fail to get block #{} !", block_number))
                .block
                .issuers()[0];
            let issuer_count_blocks = if let Some(issuer_count_blocks) = current_frame.get(&issuer)
            {
                issuer_count_blocks + 1
            } else {
                1
            };
            current_frame.insert(issuer, issuer_count_blocks);
        }
        current_frame
    })?)
}

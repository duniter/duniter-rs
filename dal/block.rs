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

use super::constants::MAX_FORKS;
use duniter_crypto::keys::*;
use duniter_documents::v10::BlockDocument;
use duniter_documents::Document;
use duniter_documents::{BlockHash, BlockId, Blockstamp, PreviousBlockstamp};
use durs_wot::NodeId;
use std::collections::HashMap;
use *;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// A block as it is saved in a database
pub struct DALBlock {
    /// Fork id
    pub fork_id: ForkId,
    /// True only if the block is on an isolated fork
    pub isolate: bool,
    /// Block document
    pub block: BlockDocument,
    /// List of certifications that expire in this block.
    /// Warning : BlockId contain the emission block, not the written block !
    /// HashMap<(Source, Target), CreatedBlockId>
    pub expire_certs: Option<HashMap<(NodeId, NodeId), BlockId>>,
}

impl DALBlock {
    /// Get blockstamp
    pub fn blockstamp(&self) -> Blockstamp {
        self.block.blockstamp()
    }
}

///Get forks status
pub fn get_forks(
    forks_db: &BinDB<ForksV10Datas>,
    current_blockstamp: Blockstamp,
) -> Result<Vec<ForkStatus>, DALError> {
    Ok(forks_db.read(|forks_db| {
        let blockchain_meta_datas = forks_db
            .get(&ForkId(0))
            .expect("Fatal error : ForksV10DB not contain local chain !");
        let mut forks = Vec::new();
        for fork_id in 1..*MAX_FORKS {
            if let Some(fork_meta_datas) = forks_db.get(&ForkId(fork_id)) {
                if fork_meta_datas.is_empty() {
                    forks.push(ForkStatus::Free());
                } else if fork_meta_datas.contains_key(&current_blockstamp) {
                    forks.push(ForkStatus::Stackable(ForkAlreadyCheck(false)));
                } else {
                    let roll_back_max = if current_blockstamp.id.0 > 101 {
                        current_blockstamp.id.0 - 101
                    } else {
                        0
                    };
                    let mut max_common_block_id = None;
                    let mut too_old = false;
                    for previous_blockstamp in fork_meta_datas.keys() {
                        if blockchain_meta_datas.contains_key(&previous_blockstamp) {
                            if previous_blockstamp.id.0 >= roll_back_max {
                                if previous_blockstamp.id.0
                                    >= max_common_block_id.unwrap_or(BlockId(0)).0
                                {
                                    max_common_block_id = Some(previous_blockstamp.id);
                                    too_old = false;
                                }
                            } else {
                                too_old = true;
                            }
                        }
                    }
                    if too_old {
                        forks.push(ForkStatus::TooOld(ForkAlreadyCheck(false)));
                    } else if let Some(max_common_block_id) = max_common_block_id {
                        forks.push(ForkStatus::RollBack(
                            ForkAlreadyCheck(false),
                            max_common_block_id,
                        ));
                    } else {
                        forks.push(ForkStatus::Isolate());
                    }
                }
            } else {
                forks.push(ForkStatus::Free());
            }
        }
        forks
    })?)
}
/// get current blockstamp
pub fn get_current_blockstamp(blocks_db: &BlocksV10DBs) -> Result<Option<Blockstamp>, DALError> {
    let current_previous_blockstamp = blocks_db.blockchain_db.read(|db| {
        let blockchain_len = db.len() as u32;
        if blockchain_len == 0 {
            None
        } else if let Some(dal_block) = db.get(&BlockId(blockchain_len - 1)) {
            if blockchain_len > 1 {
                Some(Blockstamp {
                    id: BlockId(blockchain_len - 2),
                    hash: BlockHash(dal_block.block.previous_hash),
                })
            } else {
                Some(Blockstamp::default())
            }
        } else {
            None
        }
    })?;
    if current_previous_blockstamp.is_none() {
        return Ok(None);
    }
    let current_previous_blockstamp = current_previous_blockstamp.expect("safe unwrap");
    if let Some(current_block_hash) = blocks_db.forks_db.read(|db| {
        let blockchain_meta_datas = db
            .get(&ForkId(0))
            .expect("Fatal error : ForksDB is incoherent, please reset data and resync !");
        blockchain_meta_datas
            .get(&current_previous_blockstamp)
            .cloned()
    })? {
        Ok(Some(Blockstamp {
            id: BlockId(current_previous_blockstamp.id.0 + 1),
            hash: current_block_hash,
        }))
    } else {
        Ok(None)
    }
}

/// Get block fork id
pub fn get_fork_id_of_blockstamp(
    forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
    blockstamp: &Blockstamp,
) -> Result<Option<ForkId>, DALError> {
    Ok(forks_blocks_db.read(|db| {
        if let Some(dal_block) = db.get(blockstamp) {
            Some(dal_block.fork_id)
        } else {
            None
        }
    })?)
}

impl DALBlock {
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
    /// Get fork block
    pub fn get_block_fork(
        forks_db: &BinDB<ForksV10Datas>,
        previous_blockstamp: &PreviousBlockstamp,
    ) -> Result<Option<ForkId>, DALError> {
        Ok(forks_db.read(|forks_db| {
            for (fork_id, fork_meta_datas) in forks_db {
                if fork_meta_datas.contains_key(&previous_blockstamp) {
                    return Some(*fork_id);
                }
            }
            None
        })?)
    }
    /// Get block hash
    pub fn get_block_hash(
        db: &BinDB<LocalBlockchainV10Datas>,
        block_number: BlockId,
    ) -> Result<Option<BlockHash>, DALError> {
        Ok(db.read(|db| {
            if let Some(dal_block) = db.get(&block_number) {
                dal_block.block.hash
            } else {
                None
            }
        })?)
    }
    /// Return true if the node already knows this block
    pub fn already_have_block(
        blockchain_db: &BinDB<LocalBlockchainV10Datas>,
        forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
        blockstamp: Blockstamp,
    ) -> Result<bool, DALError> {
        let already_have_block = forks_blocks_db.read(|db| db.contains_key(&blockstamp))?;
        if !already_have_block {
            Ok(blockchain_db.read(|db| {
                if let Some(dal_block) = db.get(&blockstamp.id) {
                    if dal_block.block.hash.unwrap_or_default() == blockstamp.hash {
                        return true;
                    }
                }
                false
            })?)
        } else {
            Ok(true)
        }
    }
    /// Get stackables blocks
    pub fn get_stackables_blocks(
        forks_db: &BinDB<ForksV10Datas>,
        forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
        current_blockstamp: &Blockstamp,
    ) -> Result<Vec<DALBlock>, DALError> {
        debug!("get_stackables_blocks() after {}", current_blockstamp);
        let stackables_blocks_hashs = forks_db.read(|db| {
            let mut stackables_blocks_hashs = Vec::new();
            for fork_meta_datas in db.values() {
                if let Some(block_hash) = fork_meta_datas.get(&current_blockstamp) {
                    stackables_blocks_hashs.push(*block_hash);
                }
            }
            stackables_blocks_hashs
        })?;
        let stackables_blocks = forks_blocks_db.read(|db| {
            let mut stackables_blocks = Vec::new();
            for stackable_block_hash in stackables_blocks_hashs {
                if let Some(dal_block) = db.get(&Blockstamp {
                    id: BlockId(current_blockstamp.id.0 + 1),
                    hash: stackable_block_hash,
                }) {
                    stackables_blocks.push(dal_block.clone());
                }
            }
            stackables_blocks
        })?;
        Ok(stackables_blocks)
    }
    /// Get stackables forks
    pub fn get_stackables_forks(
        db: &BinDB<ForksV10Datas>,
        current_blockstamp: &Blockstamp,
    ) -> Result<Vec<usize>, DALError> {
        Ok(db.read(|db| {
            let mut stackables_forks = Vec::new();
            for f in 0..*MAX_FORKS {
                if let Some(fork_meta_datas) = db.get(&ForkId(f)) {
                    if fork_meta_datas.get(&current_blockstamp).is_some() {
                        stackables_forks.push(f);
                    }
                }
            }
            stackables_forks
        })?)
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
    pub fn get_block_in_local_blockchain(
        db: &BinDB<LocalBlockchainV10Datas>,
        block_id: BlockId,
    ) -> Result<Option<BlockDocument>, DALError> {
        Ok(db.read(|db| {
            if let Some(dal_block) = db.get(&block_id) {
                Some(dal_block.block.clone())
            } else {
                None
            }
        })?)
    }
    /// Get current frame of calculating members
    pub fn get_current_frame(
        &self,
        db: &BinDB<LocalBlockchainV10Datas>,
    ) -> Result<HashMap<PubKey, usize>, DALError> {
        let frame_begin = self.block.number.0 - self.block.issuers_frame as u32;
        Ok(db.read(|db| {
            let mut current_frame: HashMap<PubKey, usize> = HashMap::new();
            for block_number in frame_begin..self.block.number.0 {
                let issuer = db
                    .get(&BlockId(block_number))
                    .unwrap_or_else(|| panic!("Fail to get block #{} !", block_number))
                    .block
                    .issuers()[0];
                let issuer_count_blocks =
                    if let Some(issuer_count_blocks) = current_frame.get(&issuer) {
                        issuer_count_blocks + 1
                    } else {
                        1
                    };
                current_frame.insert(issuer, issuer_count_blocks);
            }
            current_frame
        })?)
    }
    /// Compute median issuers frame
    pub fn compute_median_issuers_frame(&mut self, db: &BinDB<LocalBlockchainV10Datas>) {
        let current_frame = self
            .get_current_frame(db)
            .expect("Fatal error : fail to read LocalBlockchainV10DB !");
        if !current_frame.is_empty() {
            let mut current_frame_vec: Vec<_> = current_frame.values().cloned().collect();
            current_frame_vec.sort_unstable();

            /*// Calculate median
            let mut median_index = match self.block.issuers_count % 2 {
                1 => (self.block.issuers_count / 2) + 1,
                _ => self.block.issuers_count / 2,
            };
            if median_index >= self.block.issuers_count {
                median_index = self.block.issuers_count - 1;
            }
            self.median_frame = current_frame_vec[median_index];
            
            // Calculate second tiercile index
            let mut second_tiercile_index = match self.block.issuers_count % 3 {
                1 | 2 => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize + 1,
                _ => (self.block.issuers_count as f64 * (2.0 / 3.0)) as usize,
            };
            if second_tiercile_index >= self.block.issuers_count {
                second_tiercile_index = self.block.issuers_count - 1;
            }
            self.second_tiercile_frame = current_frame_vec[second_tiercile_index];*/
        }
    }
}

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

use crate::entities::block::DALBlock;
use crate::ForkId;
use crate::{BinDB, DALError, ForksBlocksV10Datas, ForksV10Datas, LocalBlockchainV10Datas};
use dubp_documents::Document;
use dubp_documents::{BlockHash, BlockId, PreviousBlockstamp};
use std::collections::HashMap;

/// Write DALBlock in databases
pub fn write(
    blockchain_db: &BinDB<LocalBlockchainV10Datas>,
    forks_db: &BinDB<ForksV10Datas>,
    forks_blocks_db: &BinDB<ForksBlocksV10Datas>,
    dal_block: &DALBlock,
    from_to_fork_id: Option<ForkId>,
    sync: bool,
    revert: bool,
) -> Result<(), DALError> {
    if dal_block.fork_id.0 == 0 {
        blockchain_db.write(|db| {
            if revert {
                db.remove(&dal_block.block.number);
            } else {
                db.insert(dal_block.block.number, dal_block.clone());
            }
        })?;

        if from_to_fork_id.is_some() {
            forks_blocks_db.write(|db| {
                db.remove(&dal_block.block.blockstamp());
            })?;
        }
    }
    // Move block in a fork
    if revert {
        if let Some(to_fork_id) = from_to_fork_id {
            forks_db.write(|db| {
                let previous_blockstamp = dal_block.block.previous_blockstamp();
                let mut fork_meta_datas = db.get(&to_fork_id).unwrap().clone();
                fork_meta_datas.insert(
                    previous_blockstamp,
                    dal_block
                        .block
                        .hash
                        .expect("Try to get hash of a reduce block !"),
                );
                db.insert(to_fork_id, fork_meta_datas);
            })?;
        }
    } else if let Some(from_fork_id) = from_to_fork_id {
        // Remove block in fork origin
        forks_db.write(|db| {
            let mut fork_meta_datas = db
                .get(&from_fork_id)
                .expect("from_fork_id don(t exist !")
                .clone();
            let previous_blockstamp = dal_block.block.previous_blockstamp();
            fork_meta_datas.remove(&previous_blockstamp);
            db.insert(from_fork_id, fork_meta_datas);
            if dal_block.fork_id.0 > 0 {
                let mut fork_meta_datas = db.get(&dal_block.fork_id).unwrap().clone();
                fork_meta_datas.insert(
                    previous_blockstamp,
                    dal_block
                        .block
                        .hash
                        .expect("Try to get hash of a reduce block !"),
                );
                db.insert(from_fork_id, fork_meta_datas);
            }
        })?;
    }
    if !sync {
        let mut blockchain_meta_datas: HashMap<PreviousBlockstamp, BlockHash> = forks_db
            .read(|db| db.get(&ForkId(0)).cloned())
            .expect("Get blockchain meta datas : DALError")
            .unwrap_or_else(HashMap::new);
        let block_previous_hash = if dal_block.block.number.0 == 0 {
            PreviousBlockstamp::default()
        } else {
            PreviousBlockstamp {
                id: BlockId(dal_block.block.number.0 - 1),
                hash: BlockHash(dal_block.block.previous_hash),
            }
        };
        blockchain_meta_datas.insert(block_previous_hash, dal_block.block.hash.unwrap());
        forks_db
            .write(|db| {
                db.insert(ForkId(0), blockchain_meta_datas);
            })
            .expect("Write blockchain meta datas : DALError");
    }
    Ok(())
}

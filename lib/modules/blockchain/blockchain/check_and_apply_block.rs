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

use std::collections::HashMap;

use crate::apply_valid_block::*;
use crate::verify_block::*;
use crate::*;
use dubp_documents::Document;
use dubp_documents::{BlockHash, BlockId, Blockstamp, PreviousBlockstamp};
use dup_crypto::keys::*;
use durs_blockchain_dal::entities::block::DALBlock;
use durs_blockchain_dal::*;

#[derive(Debug, Copy, Clone)]
pub enum BlockError {
    VerifyBlockHashsError(VerifyBlockHashsError),
    DALError(DALError),
    InvalidBlock(InvalidBlockError),
    ApplyValidBlockError(ApplyValidBlockError),
    NoForkAvailable(),
    UnknowError(),
}

impl From<VerifyBlockHashsError> for BlockError {
    fn from(err: VerifyBlockHashsError) -> Self {
        BlockError::VerifyBlockHashsError(err)
    }
}

impl From<DALError> for BlockError {
    fn from(err: DALError) -> Self {
        BlockError::DALError(err)
    }
}

impl From<ApplyValidBlockError> for BlockError {
    fn from(err: ApplyValidBlockError) -> Self {
        BlockError::ApplyValidBlockError(err)
    }
}

pub fn check_and_apply_block<W: WebOfTrust>(
    blocks_databases: &BlocksV10DBs,
    certs_db: &BinDB<CertsExpirV10Datas>,
    block: Block,
    current_blockstamp: &Blockstamp,
    wot_index: &mut HashMap<PubKey, NodeId>,
    wot_db: &BinDB<W>,
    forks_states: &[ForkStatus],
) -> Result<ValidBlockApplyReqs, BlockError> {
    let block_from_network = block.is_from_network();
    let block_doc: BlockDocument = block.into_doc();

    // Get BlockDocument && check if already have block
    let already_have_block = if block_from_network {
        readers::block::already_have_block(
            &blocks_databases.blockchain_db,
            &blocks_databases.forks_blocks_db,
            block_doc.blockstamp(),
        )?
    } else {
        false
    };

    // Verify block hashs
    verify_block_hashs(&block_doc)?;

    // Check block chainability
    if (block_doc.number.0 == current_blockstamp.id.0 + 1
        && block_doc.previous_hash.to_string() == current_blockstamp.hash.0.to_string())
        || (block_doc.number.0 == 0 && *current_blockstamp == Blockstamp::default())
    {
        debug!(
            "stackable_block : block {} chainable !",
            block_doc.blockstamp()
        );
        // Detect expire_certs
        let blocks_expiring = Vec::with_capacity(0);
        let expire_certs =
            durs_blockchain_dal::readers::certs::find_expire_certs(certs_db, blocks_expiring)?;

        // Try stack up block
        let old_fork_id = if block_from_network {
            durs_blockchain_dal::readers::block::get_fork_id_of_blockstamp(
                &blocks_databases.forks_blocks_db,
                &block_doc.blockstamp(),
            )?
        } else {
            None
        };

        // Verify block validity (check all protocol rule, very long !)
        verify_block_validity(
            &block_doc,
            &blocks_databases.blockchain_db,
            certs_db,
            wot_index,
            wot_db,
        )?;

        return Ok(apply_valid_block(
            block_doc,
            wot_index,
            wot_db,
            &expire_certs,
            old_fork_id,
        )?);
    } else if !already_have_block
        && (block_doc.number.0 >= current_blockstamp.id.0
            || (current_blockstamp.id.0 - block_doc.number.0) < 100)
    {
        debug!(
            "stackable_block : block {} not chainable, store this for future !",
            block_doc.blockstamp()
        );
        let (fork_id, new_fork) = writers::fork_tree::assign_fork_to_new_block(
            &blocks_databases.forks_db,
            &PreviousBlockstamp {
                id: BlockId(block_doc.number.0 - 1),
                hash: BlockHash(block_doc.previous_hash),
            },
            &block_doc
                .hash
                .expect("Try to get hash of an uncompleted or reduce block"),
        )?;
        if let Some(fork_id) = fork_id {
            let mut isolate = true;
            let fork_state = if new_fork {
                ForkStatus::Isolate()
            } else {
                forks_states[fork_id.0]
            };
            match fork_state {
                ForkStatus::Stackable(_) | ForkStatus::RollBack(_, _) | ForkStatus::TooOld(_) => {
                    isolate = false
                }
                _ => {}
            }

            let dal_block = DALBlock {
                fork_id,
                isolate,
                block: block_doc.clone(),
                expire_certs: None,
            };

            durs_blockchain_dal::writers::block::write(
                &blocks_databases.blockchain_db,
                &blocks_databases.forks_db,
                &blocks_databases.forks_blocks_db,
                &dal_block,
                None,
                false,
                false,
            )
            .expect("durs_blockchain_dal::writers::block::write() : DALError")
        } else {
            return Err(BlockError::NoForkAvailable());
        }
    } else {
        debug!(
            "stackable_block : block {} not chainable and already stored or out of forkWindowSize !",
            block_doc.blockstamp()
        );
    }
    Err(BlockError::UnknowError())
}

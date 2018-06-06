use block::DALBlock;
use duniter_documents::blockchain::Document;
use duniter_documents::{BlockHash, BlockId, Blockstamp, PreviousBlockstamp};
use std::collections::HashMap;
use ForkId;
use {BinFileDB, DALError, ForksBlocksV10Datas, ForksV10Datas, LocalBlockchainV10Datas};

/// Write DALBlock in databases
pub fn write(
    blockchain_db: &BinFileDB<LocalBlockchainV10Datas>,
    forks_db: &BinFileDB<ForksV10Datas>,
    forks_blocks_db: &BinFileDB<ForksBlocksV10Datas>,
    dal_block: &DALBlock,
    old_fork_id: Option<ForkId>,
    sync: bool,
) -> Result<(), DALError> {
    if dal_block.fork_id.0 == 0 {
        blockchain_db.write(|db| {
            db.insert(dal_block.block.number, dal_block.clone());
        })?;

        if old_fork_id.is_some() {
            forks_blocks_db.write(|db| {
                db.remove(&dal_block.block.blockstamp());
            })?;
        }
    }
    if let Some(old_fork_id) = old_fork_id {
        forks_db.write(|db| {
            let mut fork_meta_datas = db
                .get(&old_fork_id)
                .expect("old_fork_id don(t exist !")
                .clone();
            let previous_blockstamp = Blockstamp {
                id: BlockId(dal_block.block.blockstamp().id.0 - 1),
                hash: dal_block
                    .block
                    .hash
                    .expect("Try to get hash of an uncompleted or reduce block !"),
            };
            fork_meta_datas.remove(&previous_blockstamp);
            db.insert(old_fork_id, fork_meta_datas);
            if dal_block.fork_id.0 > 0 {
                let mut fork_meta_datas = db.get(&dal_block.fork_id).unwrap().clone();
                fork_meta_datas.insert(previous_blockstamp, dal_block.block.hash.unwrap());
                db.insert(old_fork_id, fork_meta_datas);
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

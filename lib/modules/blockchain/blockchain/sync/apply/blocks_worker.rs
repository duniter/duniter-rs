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

use crate::sync::*;
use pbr::ProgressBar;
use std::sync::mpsc;

pub fn execute(
    pool: &ThreadPool,
    sender_sync_thread: mpsc::Sender<MessForSyncThread>,
    recv: mpsc::Receiver<SyncJobsMess>,
    databases: BlocksV10DBs,
    mut apply_pb: ProgressBar<std::io::Stdout>,
) {
    // Launch blocks_worker thread
    pool.execute(move || {
        let blocks_job_begin = SystemTime::now();

        // Listen db requets
        let mut chunk_index = 0;
        let mut blockchain_meta_datas = HashMap::new();
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::BlocksDBsWriteQuery(req)) = recv.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            // Apply db request
            req.apply(&databases, true)
                .expect("Fatal error : Fail to apply DBWriteRequest !");
            if let BlocksDBsWriteQuery::WriteBlock(
                ref _dal_block,
                ref _old_fork_id,
                ref previous_blockstamp,
                ref previous_hash,
            ) = req
            {
                blockchain_meta_datas.insert(*previous_blockstamp, *previous_hash);
                chunk_index += 1;
                if chunk_index == 250 {
                    chunk_index = 0;
                    apply_pb.inc();
                }
            }
            wait_begin = SystemTime::now();
        }

        // Indexing blockchain meta datas
        info!("Indexing blockchain meta datas...");
        /*let blockchain_meta_datas: HashMap<PreviousBlockstamp, BlockHash> = databases
        .blockchain_db
        .read(|db| {
            let mut blockchain_meta_datas: HashMap<
                PreviousBlockstamp,
                BlockHash,
            > = HashMap::new();
            for dal_block in db.values() {
                let block_previous_hash = if dal_block.block.number.0 == 0 {
                    PreviousBlockstamp::default()
                } else {
                    PreviousBlockstamp {
                        id: BlockId(dal_block.block.number.0 - 1),
                        hash: BlockHash(dal_block.block.previous_hash),
                    }
                };
                blockchain_meta_datas
                    .insert(block_previous_hash, dal_block.block.expect("Try to get hash of an uncompleted or reduce block !"));
            }
            blockchain_meta_datas
        })
        .expect("Indexing blockchain meta datas : DALError");*/
        databases
            .forks_db
            .write(|db| {
                db.insert(ForkId(0), blockchain_meta_datas);
            })
            .expect("Indexing blockchain meta datas : DALError");

        // Increment progress bar (last chunk)
        apply_pb.inc();
        // Save blockchain, and fork databases
        println!();
        println!("Write indexs in files...");
        info!("Save blockchain and forks databases in files...");
        databases.save_dbs();

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let blocks_job_duration =
            SystemTime::now().duration_since(blocks_job_begin).unwrap() - all_wait_duration;
        info!(
            "blocks_job_duration={},{:03} seconds.",
            blocks_job_duration.as_secs(),
            blocks_job_duration.subsec_millis()
        );
    });
}

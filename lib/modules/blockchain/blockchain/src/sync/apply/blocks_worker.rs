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
    blocks_dbs: BlocksV10DBs,
    forks_db: ForksDBs,
    target_blockstamp: Blockstamp,
    mut apply_pb: ProgressBar<std::io::Stdout>,
) {
    // Launch blocks_worker thread
    pool.execute(move || {
        let blocks_job_begin = SystemTime::now();

        // Listen db requets
        let mut chunk_index = 0;
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::BlocksDBsWriteQuery(req)) = recv.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();

            // Apply db request
            req.apply(
                &blocks_dbs.blockchain_db,
                &forks_db,
                200, // TODO replace by fork_window_size
                Some(target_blockstamp),
            )
            .expect("Fatal error : Fail to apply DBWriteRequest !");

            chunk_index += 1;
            if chunk_index == 250 {
                chunk_index = 0;
                apply_pb.inc();
            }
            wait_begin = SystemTime::now();
        }

        // Increment progress bar (last chunk)
        apply_pb.inc();
        // Save blockchain, and fork databases
        println!();
        println!("Write indexs in files...");
        info!("Save blockchain and forks databases in files...");
        blocks_dbs.save_dbs();
        forks_db.save_dbs();

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

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

use crate::sync::*;
use durs_bc_db_reader::BcDbRead;
use pbr::ProgressBar;

pub fn execute(
    pool: &ThreadPool,
    sender_sync_thread: Sender<MessForSyncThread>,
    recv: Receiver<SyncJobsMess>,
    db: Db,
    target_blockstamp: Blockstamp,
    mut apply_pb: ProgressBar<std::io::Stdout>,
) {
    // Launch blocks_worker thread
    pool.execute(move || {
        let blocks_job_begin = Instant::now();

        // Get fork tree
        let mut fork_tree = db
            .r(|db_r| durs_bc_db_reader::current_metadata::get_fork_tree(db_r))
            .expect("Fail to read DB.");

        // Listen db requets
        let mut chunk_index = 0;
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = Instant::now();

        if let Ok(SyncJobsMess::ForkWindowSize(fork_window_size)) = recv.recv() {
            log::info!(
                "Block worker receive fork_window_size={}.",
                fork_window_size
            );
            loop {
                match recv.recv() {
                    Ok(SyncJobsMess::BlocksDBsWriteQuery(req)) => {
                        all_wait_duration += wait_begin.elapsed();

                        // Apply db request
                        db.write(|mut w| {
                            req.apply(
                                &db,
                                &mut w,
                                &mut fork_tree,
                                fork_window_size,
                                Some(target_blockstamp),
                            )?;
                            Ok(WriteResp::from(w))
                        })
                        .expect("Fatal error : Fail to apply BlocksDBsWriteQuery !");

                        chunk_index += 1;
                        if chunk_index == 250 {
                            chunk_index = 0;
                            apply_pb.inc();
                        }
                        wait_begin = Instant::now();
                    }
                    Ok(SyncJobsMess::End) | Err(_) => {
                        log::info!("Sync: block worker channel closed.");
                        break;
                    }
                    Ok(msg) => fatal_error!(
                        "Dev error: block worker receive unexpected message: {:?}",
                        msg
                    ),
                }
            }
        } else {
            fatal_error!("Dev error: block worker must first receive fork window size")
        }

        // Increment progress bar (last chunk)
        apply_pb.inc();
        // Save fork tree
        println!();
        println!("Write indexs in files...");
        info!("Save db...");
        db.write(|mut w| {
            durs_bc_db_writer::blocks::fork_tree::save_fork_tree(&db, &mut w, &fork_tree)?;
            Ok(WriteResp::from(w))
        })
        .unwrap_or_else(|_| fatal_error!("DB corrupted, please reset data."));

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish(Some(db)))
            .expect("Fatal error : sync_thread unrechable !");
        let blocks_job_duration = blocks_job_begin.elapsed() - all_wait_duration;
        info!(
            "blocks_job_duration={},{:03} seconds.",
            blocks_job_duration.as_secs(),
            blocks_job_duration.subsec_millis()
        );
    });
}

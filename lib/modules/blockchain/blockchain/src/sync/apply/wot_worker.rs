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
use std::ops::Deref;

pub fn execute(
    pool: &ThreadPool,
    profile_path: PathBuf,
    sender_sync_thread: Sender<MessForSyncThread>,
    recv: Receiver<SyncJobsMess>,
) {
    // Launch wot_worker thread
    pool.execute(move || {
        let wot_job_begin = Instant::now();
        // Open databases
        let db_path = durs_conf::get_blockchain_db_path(profile_path);
        let db = open_db(&db_path).expect("Fail to open DB.");

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = Instant::now();
        while let Ok(mess) = recv.recv() {
            all_wait_duration += wait_begin.elapsed();
            match mess {
                SyncJobsMess::WotsDBsWriteQuery(blockstamp, currency_params, req) => {
                    db.write(|mut w| {
                        req.apply(&db, &mut w, &blockstamp, &currency_params.deref())?;
                        Ok(WriteResp::from(w))
                    })
                    .unwrap_or_else(|_| {
                        fatal_error!("Fail to apply WotsDBsWriteQuery ({})", blockstamp)
                    });
                }
                SyncJobsMess::End => break,
                _ => {}
            }
            wait_begin = Instant::now();
        }

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish(None))
            .expect("Fatal error : sync_thread unrechable !");
        let wot_job_duration = wot_job_begin.elapsed() - all_wait_duration;
        info!(
            "wot_job_duration={},{:03} seconds.",
            wot_job_duration.as_secs(),
            wot_job_duration.subsec_millis()
        );
    });
}

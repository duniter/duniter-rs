//  Copyright (C) 2018  The Dunitrust Project Developers.
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
use std::sync::mpsc;

pub fn execute(
    pool: &ThreadPool,
    profile_path: PathBuf,
    sender_sync_thread: mpsc::Sender<MessForSyncThread>,
    recv: mpsc::Receiver<SyncJobsMess>,
) {
    // Launch wot_worker thread
    pool.execute(move || {
        let wot_job_begin = SystemTime::now();
        // Open databases
        let db_path = durs_conf::get_blockchain_db_path(profile_path);
        let databases = WotsV10DBs::open(Some(&db_path));

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(mess) = recv.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            match mess {
                SyncJobsMess::WotsDBsWriteQuery(blockstamp, currency_params, req) => req
                    .apply(&blockstamp, &currency_params.deref(), &databases)
                    .expect("Fatal error : Fail to apply DBWriteRequest !"),
                SyncJobsMess::End => break,
                _ => {}
            }
            wait_begin = SystemTime::now();
        }
        // Save wots databases
        info!("Save wots databases in files...");
        databases.save_dbs_except_graph();

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish())
            .expect("Fatal error : sync_thread unrechable !");
        let wot_job_duration =
            SystemTime::now().duration_since(wot_job_begin).unwrap() - all_wait_duration;
        info!(
            "wot_job_duration={},{:03} seconds.",
            wot_job_duration.as_secs(),
            wot_job_duration.subsec_millis()
        );
    });
}

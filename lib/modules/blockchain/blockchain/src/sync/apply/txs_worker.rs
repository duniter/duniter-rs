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

pub fn execute(
    pool: &ThreadPool,
    profile_path: PathBuf,
    sender_sync_thread: Sender<MessForSyncThread>,
    recv: Receiver<SyncJobsMess>,
) {
    // Launch tx_worker thread
    pool.execute(move || {
        let tx_job_begin = SystemTime::now();
        // Open databases
        let db_path = durs_conf::get_blockchain_db_path(profile_path);
        let db = open_db(db_path.as_path()).expect("Fail to open blockchain DB.");

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::CurrencyDBsWriteQuery {
            in_fork_window,
            req,
        }) = recv.recv()
        {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            // Apply db request
            db.write(|mut w| {
                req.apply(&db, &mut w, None, in_fork_window)?;
                Ok(w)
            })
            .expect("Fatal error : Fail to apply CurrencyDBsWriteQuery !");
            wait_begin = SystemTime::now();
        }

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish(None))
            .expect("Fatal error : sync_thread unrechable !");
        let tx_job_duration =
            SystemTime::now().duration_since(tx_job_begin).unwrap() - all_wait_duration;
        info!(
            "tx_job_duration={},{:03} seconds.",
            tx_job_duration.as_secs(),
            tx_job_duration.subsec_millis()
        );
    });
}

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
use std::sync::mpsc;

pub fn execute(
    pool: &ThreadPool,
    profile: String,
    currency: CurrencyName,
    sender_sync_thread: mpsc::Sender<MessForSyncThread>,
    recv: mpsc::Receiver<SyncJobsMess>,
) {
    // Launch tx_worker thread
    pool.execute(move || {
        let tx_job_begin = SystemTime::now();
        // Open databases
        let db_path = durs_conf::get_blockchain_db_path(&profile, &currency);
        let databases = CurrencyV10DBs::open(Some(&db_path));

        // Listen db requets
        let mut all_wait_duration = Duration::from_millis(0);
        let mut wait_begin = SystemTime::now();
        while let Ok(SyncJobsMess::CurrencyDBsWriteQuery(req)) = recv.recv() {
            all_wait_duration += SystemTime::now().duration_since(wait_begin).unwrap();
            // Apply db request
            req.apply(&databases)
                .expect("Fatal error : Fail to apply DBWriteRequest !");
            wait_begin = SystemTime::now();
        }
        // Save tx, utxo, du and balances databases
        info!("Save tx and sources database in file...");
        databases.save_dbs(true, true);

        // Send finish signal
        sender_sync_thread
            .send(MessForSyncThread::ApplyFinish())
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

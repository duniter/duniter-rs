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

#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
use dubp_block_doc::BlockDocument;
use dubp_blocks_tests_tools::mocks::gen_empty_block_v10_with_issuer_and_pow_min as gen_empty_block;
use dubp_common_doc::BlockNumber;
use durs_bc_db_writer::blocks::insert_new_head_block;
use durs_bc_db_writer::{Db, DbError};
use durs_wot::WotId;
use tempfile::tempdir;

mod common;

const INITIAL_POW_MIN: usize = 70;

fn insert_250_blocks_with_distinct_issuers(db: &Db) -> Result<(), DbError> {
    for i in 0..250 {
        let byte = (i % 255) as u8;
        let pubkey = dup_crypto_tests_tools::mocks::pubkey_from_byte(byte);
        let block = BlockDocument::V10(gen_empty_block(
            BlockNumber(i as u32),
            pubkey,
            INITIAL_POW_MIN + (i % 25),
        ));
        db.write(|mut w| {
            common::insert_wot_index_entry(&db, &mut w, WotId(i), pubkey)?;
            insert_new_head_block(&db, &mut w, None, common::to_db_block(block))?;
            Ok(w)
        })?;
    }
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    // Open temporary database
    let db = durs_bc_db_writer::open_db(tempdir().expect("Fail open tmp db").path())
        .expect("Fail open tmp db");

    c.bench_function("insert 250 blocks", |b| {
        b.iter(|| insert_250_blocks_with_distinct_issuers(black_box(&db)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

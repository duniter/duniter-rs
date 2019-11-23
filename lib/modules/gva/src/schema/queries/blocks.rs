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

// ! Module execute GraphQl schema blocks query

use crate::schema::entities::block::Block;
use crate::schema::inputs::paging::FilledPaging;
use crate::schema::BlockInterval;
use crate::schema::Paging;
use dubp_common_doc::BlockNumber;
use durs_bc_db_reader::blocks::DbBlock;
use durs_bc_db_reader::{BcDbRoTrait, DbError};
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute<DB: BcDbRoTrait>(
    db: &DB,
    _trail: &QueryTrail<'_, Block, Walked>,
    paging_opt: Option<Paging>,
    block_interval_opt: Option<BlockInterval>,
    step: usize,
) -> Result<Vec<Block>, DbError> {
    // Get interval
    let interval = BlockInterval::get_range(db, block_interval_opt)?;

    // Get blocks numbers that respect filters
    let blocks_numbers: Vec<BlockNumber> =
        interval.clone().map(|i| BlockNumber(i as u32)).collect(); // TODO

    // Apply interval
    let blocks_numbers: Vec<BlockNumber> = blocks_numbers
        .into_iter()
        .filter(|n| interval.contains(&(n.0 as usize)))
        .collect();
    let count = blocks_numbers.len();

    // Apply paging and step
    let paging = FilledPaging::from(paging_opt);
    let page_range = paging.get_page_range(count, step);
    let blocks_numbers: Vec<BlockNumber> = page_range
        .step_by(step)
        .map(|i| blocks_numbers[i])
        .collect();

    // Get blocks
    let blocks: Vec<DbBlock> = db.get_db_blocks_in_local_blockchain(blocks_numbers)?;

    Ok(blocks.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod tests {
    use crate::db::BcDbRo;
    use crate::schema::queries::tests;
    use dubp_block_doc::block::v10::BlockDocumentV10;
    use dubp_block_doc::block::BlockDocument;
    use dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10;
    use dubp_common_doc::traits::Document;
    use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
    use dup_crypto::hashs::Hash;
    use dup_crypto_tests_tools::mocks::{hash, pubkey};
    use durs_bc_db_reader::blocks::DbBlock;
    use mockall::predicate::eq;
    use serde_json::json;

    fn block_0() -> BlockDocumentV10 {
        let mut block_0 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(0),
                hash: BlockHash(hash('A')),
            },
            1_488_987_127,
            Hash::default(),
        );
        block_0.issuers = vec![pubkey('A')];
        block_0
    }
    fn block_0_json() -> serde_json::Value {
        json!({
            "commonTime": 1_488_987_127.0,
            "currency": "test_currency",
            "hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "issuer": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "number": 0,
            "version": 10
        })
    }

    fn block_1() -> BlockDocumentV10 {
        let mut block_1 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(1),
                hash: BlockHash(hash('B')),
            },
            1_488_987_128,
            Hash::default(),
        );
        block_1.issuers = vec![pubkey('B')];
        block_1
    }
    fn block_1_json() -> serde_json::Value {
        json!({
            "commonTime": 1_488_987_128.0,
            "currency": "test_currency",
            "hash": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            "issuer": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            "number": 1,
            "version": 10
        })
    }

    fn block_2() -> BlockDocumentV10 {
        let mut block_2 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(2),
                hash: BlockHash(hash('C')),
            },
            1_488_987_129,
            Hash::default(),
        );
        block_2.issuers = vec![pubkey('C')];
        block_2
    }
    fn block_2_json() -> serde_json::Value {
        json!({
            "commonTime": 1_488_987_129.0,
            "currency": "test_currency",
            "hash": "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
            "issuer": "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
            "number": 2,
            "version": 10
        })
    }

    fn block_3() -> BlockDocumentV10 {
        let mut block_3 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(3),
                hash: BlockHash(hash('D')),
            },
            1_488_987_130,
            Hash::default(),
        );
        block_3.issuers = vec![pubkey('D')];
        block_3
    }
    fn block_3_json() -> serde_json::Value {
        json!({
            "commonTime": 1_488_987_130.0,
            "currency": "test_currency",
            "hash": "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
            "issuer": "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD",
            "number": 3,
            "version": 10
        })
    }

    fn block_4() -> BlockDocumentV10 {
        let mut block_4 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(4),
                hash: BlockHash(hash('E')),
            },
            1_488_987_131,
            Hash::default(),
        );
        block_4.issuers = vec![pubkey('E')];
        block_4
    }
    fn block_4_json() -> serde_json::Value {
        json!({
            "commonTime": 1_488_987_131.0,
            "currency": "test_currency",
            "hash": "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
            "issuer": "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
            "number": 4,
            "version": 10
        })
    }

    static mut DB_TEST_BLOCKS_FROM_2: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_blocks_from_2() {
        let mut mock_db = BcDbRo::new();

        let block_2 = block_2();
        let block_3 = block_3();
        let current_block = block_4();

        let current_blockstamp = current_block.blockstamp();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(move || Ok(Some(current_blockstamp)));

        mock_db
            .expect_get_db_blocks_in_local_blockchain()
            .with(eq(vec![BlockNumber(2), BlockNumber(3), BlockNumber(4)]))
            .returning(move |_| {
                Ok(vec![
                    DbBlock {
                        block: BlockDocument::V10(block_2.clone()),
                        expire_certs: None,
                    },
                    DbBlock {
                        block: BlockDocument::V10(block_3.clone()),
                        expire_certs: None,
                    },
                    DbBlock {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS_FROM_2 });

        tests::test_gql_query(
            schema,
            "{ blocks(interval: { from: 2 }) { commonTime, currency, hash, issuer, number, version } }",
            json!({
                "data": {
                    "blocks": [
                        block_2_json(),
                        block_3_json(),
                        block_4_json(),
                    ]
                }
            }),
        );
    }

    static mut DB_TEST_BLOCKS_STEP_2: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_blocks_with_step_2() {
        let mut mock_db = BcDbRo::new();

        let block_0 = block_0();
        let current_block = block_2();

        let current_blockstamp = current_block.blockstamp();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(move || Ok(Some(current_blockstamp)));

        mock_db
            .expect_get_db_blocks_in_local_blockchain()
            .with(eq(vec![BlockNumber(0), BlockNumber(2)]))
            .returning(move |_| {
                Ok(vec![
                    DbBlock {
                        block: BlockDocument::V10(block_0.clone()),
                        expire_certs: None,
                    },
                    DbBlock {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS_STEP_2 });

        tests::test_gql_query(
            schema,
            "{ blocks(step: 2) { commonTime, currency, hash, issuer, number, version } }",
            json!({
                "data": {
                    "blocks": [{
                        "commonTime": 1_488_987_127.0,
                        "currency": "test_currency",
                        "hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "issuer": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "number": 0,
                        "version": 10
                    },
                    {
                        "commonTime": 1_488_987_129.0,
                        "currency": "test_currency",
                        "hash": "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                        "issuer": "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                        "number": 2,
                        "version": 10
                    }]
                }
            }),
        );
    }

    static mut DB_TEST_BLOCKS: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_blocks() {
        let mut mock_db = BcDbRo::new();

        let block_0 = block_0();
        let block_1 = block_1();
        let current_block = block_2();

        let current_blockstamp = current_block.blockstamp();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(move || Ok(Some(current_blockstamp)));

        mock_db
            .expect_get_db_blocks_in_local_blockchain()
            .with(eq(vec![BlockNumber(0), BlockNumber(1), BlockNumber(2)]))
            .returning(move |_| {
                Ok(vec![
                    DbBlock {
                        block: BlockDocument::V10(block_0.clone()),
                        expire_certs: None,
                    },
                    DbBlock {
                        block: BlockDocument::V10(block_1.clone()),
                        expire_certs: None,
                    },
                    DbBlock {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS });

        tests::test_gql_query(
            schema,
            "{ blocks { commonTime, currency, hash, issuer, number, version } }",
            json!({
                "data": {
                    "blocks": [
                    block_0_json(),
                    block_1_json(),
                    block_2_json()]
                }
            }),
        );
    }
}

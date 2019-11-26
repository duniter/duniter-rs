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
use crate::schema::entities::blocks_page::BlocksPage;
use crate::schema::inputs::block_interval::{BlockInterval, FilledBlockInterval};
use crate::schema::inputs::paging::{FilledPaging, Paging};
use crate::schema::inputs::sort_order::SortOrder;
use dubp_common_doc::BlockNumber;
use durs_bc_db_reader::blocks::BlockDb;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute<DB: BcDbInReadTx>(
    db: &DB,
    trail: &QueryTrail<'_, BlocksPage, Walked>,
    paging_opt: Option<Paging>,
    block_interval_opt: Option<BlockInterval>,
    step: usize,
    sort_order: SortOrder,
) -> Result<BlocksPage, DbError> {
    // Get current block number opt
    let current_block_number_opt = if let Some(current_blockstamp) = db.get_current_blockstamp()? {
        Some(current_blockstamp.id)
    } else {
        None
    };

    // Get interval
    let interval =
        FilledBlockInterval::new(block_interval_opt, current_block_number_opt).get_range();

    // Get blocks numbers that respect filters
    let blocks_numbers: Vec<BlockNumber> =
        interval.clone().map(|i| BlockNumber(i as u32)).collect(); // TODO

    // Apply interval
    let mut blocks_numbers: Vec<BlockNumber> = blocks_numbers
        .into_iter()
        .filter(|n| interval.contains(&(n.0 as usize)))
        .collect();
    let total_blocks_count = blocks_numbers.len();

    // Apply sort
    if let SortOrder::Desc = sort_order {
        blocks_numbers = blocks_numbers.into_iter().rev().collect();
    }

    // Apply paging and step
    let paging = FilledPaging::from(paging_opt);
    let (page_range, count_pages) = paging.get_page_range(total_blocks_count, step);
    let blocks_numbers_len = blocks_numbers.len();
    let blocks_numbers: Vec<BlockNumber> = page_range
        .step_by(step)
        .filter_map(|i| {
            if i < blocks_numbers_len {
                Some(blocks_numbers[i])
            } else {
                None
            }
        })
        .collect();

    // Get blocks
    let blocks: Vec<BlockDb> = db.get_db_blocks_in_local_blockchain(blocks_numbers)?;

    // Convert BlockDb (db entity) into Block (gva entity)
    let ask_field_issuer_name = BlocksPage::ask_field_blocks_issuer_name(trail);
    let blocks: Vec<Block> = blocks
        .into_iter()
        .map(|block_db| Block::from_block_db(db, block_db, ask_field_issuer_name))
        .collect::<Result<Vec<Block>, DbError>>()?;

    Ok(BlocksPage {
        blocks,
        current_page_number: paging.page_number as i32,
        interval_from: *interval.start() as i32,
        interval_to: *interval.end() as i32,
        last_page_number: (count_pages - 1) as i32,
        total_blocks_count: (interval.end() - interval.start() + 1) as i32,
    })
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
    use durs_bc_db_reader::blocks::BlockDb;
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
                    BlockDb {
                        block: BlockDocument::V10(block_2.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(block_3.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS_FROM_2 });

        tests::test_gql_query(
            schema,
            "{ blocks(interval: { from: 2 }) {
                blocks { commonTime, currency, hash, issuer, number, version },
                currentPageNumber, intervalFrom, intervalTo, lastPageNumber, totalBlocksCount
            } }",
            json!({
                "data": {
                    "blocks": {
                        "blocks": [
                            block_2_json(),
                            block_3_json(),
                            block_4_json(),
                        ],
                        "currentPageNumber": 0,
                        "intervalFrom": 2,
                        "intervalTo": 4,
                        "lastPageNumber": 0,
                        "totalBlocksCount": 3,
                    }
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
                    BlockDb {
                        block: BlockDocument::V10(block_0.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS_STEP_2 });

        tests::test_gql_query(
            schema,
            "{ blocks(step: 2) {
                blocks { commonTime, currency, hash, issuer, number, version },
                currentPageNumber, intervalFrom, intervalTo, lastPageNumber, totalBlocksCount
            } }",
            json!({
                "data": {
                    "blocks": {
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
                        }],
                        "currentPageNumber": 0,
                        "intervalFrom": 0,
                        "intervalTo": 2,
                        "lastPageNumber": 0,
                        "totalBlocksCount": 3
                    }
                }
            }),
        );
    }

    static mut DB_TEST_BLOCKS_DESC: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_blocks_order_desc() {
        let mut mock_db = BcDbRo::new();

        let current_block = block_2();
        let block_1 = block_1();
        let block_0 = block_0();

        let current_blockstamp = current_block.blockstamp();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(move || Ok(Some(current_blockstamp)));

        mock_db
            .expect_get_db_blocks_in_local_blockchain()
            .with(eq(vec![BlockNumber(2), BlockNumber(1), BlockNumber(0)]))
            .returning(move |_| {
                Ok(vec![
                    BlockDb {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(block_1.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(block_0.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let global_context = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS_DESC });

        tests::test_gql_query(
            global_context,
            "{ blocks(sortOrder: DESC) {
                blocks { commonTime, currency, hash, issuer, number, version },
                currentPageNumber, intervalFrom, intervalTo, lastPageNumber, totalBlocksCount
            } }",
            json!({
                "data": {
                    "blocks": {
                        "blocks": [
                            block_2_json(),
                            block_1_json(),
                            block_0_json()
                        ],
                        "currentPageNumber": 0,
                        "intervalFrom": 0,
                        "intervalTo": 2,
                        "lastPageNumber": 0,
                        "totalBlocksCount": 3
                    }
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
                    BlockDb {
                        block: BlockDocument::V10(block_0.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(block_1.clone()),
                        expire_certs: None,
                    },
                    BlockDb {
                        block: BlockDocument::V10(current_block.clone()),
                        expire_certs: None,
                    },
                ])
            });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_BLOCKS });

        tests::test_gql_query(
            schema,
            "{ blocks {
                blocks { commonTime, currency, hash, issuer, number, version },
                currentPageNumber, intervalFrom, intervalTo, lastPageNumber, totalBlocksCount
            } }",
            json!({
                "data": {
                    "blocks": {
                        "blocks": [
                            block_0_json(),
                            block_1_json(),
                            block_2_json()
                        ],
                        "currentPageNumber": 0,
                        "intervalFrom": 0,
                        "intervalTo": 2,
                        "lastPageNumber": 0,
                        "totalBlocksCount": 3
                    }
                }
            }),
        );
    }
}

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

use super::db_err_to_juniper_err;
use crate::context::Context;
use crate::db::BcDbTrait;
use crate::schema::entities::block::Block;
use crate::schema::paging;
use crate::schema::Paging;
use durs_bc_db_reader::blocks::DbBlock;
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute(
    executor: &Executor<'_, Context>,
    _trail: &QueryTrail<'_, Block, Walked>,
    paging_opt: Option<Paging>,
) -> FieldResult<Vec<Block>> {
    let db = executor.context().get_db();

    let blocks: Vec<DbBlock> = db
        .get_db_blocks_in_local_blockchain(
            paging::FilledPaging::new(db, paging_opt)
                .map_err(db_err_to_juniper_err)?
                .get_range(),
        )
        .map_err(db_err_to_juniper_err)?;

    Ok(blocks.into_iter().map(Block::from_db_block).collect())

    /*let db: &BcDbRo = &executor.context().get_db();
    db.read(|r| {
        paging::FilledPaging::new(db, paging_opt)?
            .get_range()
            .filter_map(|n| match block::get_block(db, r, BlockNumber(n)) {
                Ok(Some(db_block)) => Some(Ok(db_block)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<Result<Vec<Block>, DbError>>()
    })
    .map_err(db_err_to_juniper_err)*/
}

#[cfg(test)]
mod tests {
    use crate::db::MockBcDbTrait;
    use crate::schema::queries::tests;
    use dubp_block_doc::block::BlockDocument;
    use dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10;
    use dubp_common_doc::traits::Document;
    use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
    use dup_crypto::hashs::Hash;
    use dup_crypto_tests_tools::mocks::{hash, pubkey};
    use durs_bc_db_reader::blocks::DbBlock;
    use mockall::predicate::eq;
    use serde_json::json;
    use std::ops::Range;

    #[test]
    fn test_graphql_blocks() {
        let mut mock_db = MockBcDbTrait::new();

        let mut block_0 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(0),
                hash: BlockHash(hash('A')),
            },
            1_488_987_127,
            Hash::default(),
        );
        block_0.issuers = vec![pubkey('A')];
        let mut block_1 = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(1),
                hash: BlockHash(hash('B')),
            },
            1_488_987_128,
            Hash::default(),
        );
        block_1.issuers = vec![pubkey('B')];
        let mut current_block = gen_empty_timed_block_v10(
            Blockstamp {
                id: BlockNumber(2),
                hash: BlockHash(hash('C')),
            },
            1_488_987_129,
            Hash::default(),
        );
        current_block.issuers = vec![pubkey('C')];

        let current_blockstamp = current_block.blockstamp();
        mock_db
            .expect_get_current_blockstamp()
            .times(1)
            .returning(move || Ok(Some(current_blockstamp)));

        mock_db
            .expect_get_db_blocks_in_local_blockchain()
            .with(eq(Range { start: 0, end: 3 }))
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

        let schema = tests::setup(mock_db);

        tests::test_gql_query(
            schema,
            "{ blocks { commonTime, currency, hash, issuer, number, version } }",
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
                        "commonTime": 1_488_987_128.0,
                        "currency": "test_currency",
                        "hash": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                        "issuer": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                        "number": 1,
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
}

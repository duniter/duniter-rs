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

// ! Module execute GraphQl schema block query

use super::db_err_to_juniper_err;
use crate::context::Context;
use crate::db::BcDbTrait;
use crate::schema::entities::block::Block;
use dubp_common_doc::BlockNumber;
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute(
    executor: &Executor<'_, Context>,
    _trail: &QueryTrail<'_, Block, Walked>,
    number: i32,
) -> FieldResult<Option<Block>> {
    let block_number = if number >= 0 {
        BlockNumber(number as u32)
    } else {
        return Err(juniper::FieldError::from("Block number must be positive."));
    };

    executor
        .context()
        .get_db()
        .get_db_block_in_local_blockchain(block_number)
        .map_err(db_err_to_juniper_err)
        .map(|db_block_opt| db_block_opt.map(Block::from_db_block))
}

#[cfg(test)]
mod tests {
    use crate::db::MockBcDbTrait;
    use crate::schema::queries::tests;
    use dubp_block_doc::block::BlockDocument;
    use dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10;
    use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
    use dup_crypto::hashs::Hash;
    use dup_crypto_tests_tools::mocks::{hash, pubkey};
    use durs_bc_db_reader::blocks::DbBlock;
    use mockall::predicate::eq;
    use serde_json::json;

    #[test]
    fn test_graphql_block() {
        let mut mock_db = MockBcDbTrait::new();
        mock_db
            .expect_get_db_block_in_local_blockchain()
            .with(eq(BlockNumber(42)))
            .returning(|_| {
                let mut block = gen_empty_timed_block_v10(
                    Blockstamp {
                        id: BlockNumber(42),
                        hash: BlockHash(hash('A')),
                    },
                    1_488_987_127,
                    Hash::default(),
                );
                block.issuers = vec![pubkey('B')];
                Ok(Some(DbBlock {
                    block: BlockDocument::V10(block),
                    expire_certs: None,
                }))
            });

        let schema = tests::setup(mock_db);

        tests::test_gql_query(
            schema.clone(),
            "{ block { commonTime, currency, hash, issuer, number, version } }",
            json!({
                "errors": [{
                    "message": "Field \"block\" argument \"number\" of type \"Int!\" is required but not provided",
                    "locations": [{
                        "line": 1,
                        "column": 3,
                    }]
                }]
            }),
        );

        tests::test_gql_query(
            schema,
            "{ block(number: 42) { commonTime, currency, hash, issuer, number, version } }",
            json!({
                "data": {
                    "block": {
                        "commonTime": 1_488_987_127.0,
                        "currency": "test_currency",
                        "hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "issuer": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                        "number": 42,
                        "version": 10
                    }
                }
            }),
        );
    }
}

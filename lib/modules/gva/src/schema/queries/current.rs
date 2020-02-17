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

// ! Module execute GraphQl schema current query

use crate::schema::entities::block::Block;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute<DB: BcDbInReadTx>(
    db: &DB,
    trail: &QueryTrail<'_, Block, Walked>,
) -> Result<Option<Block>, DbError> {
    let ask_field_issuer_name = Block::ask_field_issuer_name(trail);
    db.get_current_block()?
        .map(|block_db| Block::from_block_db(db, block_db, ask_field_issuer_name))
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::db::BcDbRo;
    use crate::schema::queries::tests;
    use dubp_block_doc::block::BlockDocument;
    use dubp_blocks_tests_tools::mocks::gen_empty_timed_block_v10;
    use dubp_common_doc::{BlockHash, BlockNumber, Blockstamp};
    use dup_crypto::hashs::Hash;
    use dup_crypto_tests_tools::mocks::{hash, pubkey};
    use durs_bc_db_reader::blocks::BlockDb;
    use durs_common_tools::UsizeSer32;
    use mockall::predicate::eq;
    use serde_json::json;

    static mut DB_TEST_CURRENT_1: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_current() {
        let mut mock_db = BcDbRo::new();
        mock_db.expect_get_current_block().times(1).returning(|| {
            let mut current_block = gen_empty_timed_block_v10(
                Blockstamp {
                    id: BlockNumber(42),
                    hash: BlockHash(hash('A')),
                },
                1_488_987_127,
                Hash::default(),
            );
            current_block.issuers = vec![pubkey('B')];
            current_block.pow_min = UsizeSer32(70);
            current_block.members_count = UsizeSer32(59);
            Ok(Some(BlockDb {
                block: BlockDocument::V10(current_block),
                expire_certs: None,
            }))
        });
        mock_db
            .expect_get_uid_from_pubkey()
            .times(1)
            .with(eq(pubkey('B')))
            .returning(|_| Ok(Some("issuerName".to_owned())));

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_CURRENT_1 });

        tests::test_gql_query(
            schema,
            "{ current { blockchainTime, currency, hash, issuer, issuerName, membersCount, number, powMin, version } }",
            json!({
                "data": {
                    "current": {
                        "blockchainTime": 1_488_987_127.0,
                        "currency": "test_currency",
                        "hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "issuer": "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
                        "issuerName": "issuerName",
                        "membersCount": 59, 
                        "number": 42,
                        "powMin": 70,
                        "version": 10
                    }
                }
            }),
        )
    }
}

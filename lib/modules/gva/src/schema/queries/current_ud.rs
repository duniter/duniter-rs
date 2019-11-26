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

// ! Module execute GraphQl schema currentUd query
use crate::schema::entities::current_ud::CurrentUd;
use durs_bc_db_reader::{BcDbInReadTx, DbError};
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute<DB: BcDbInReadTx>(
    db: &DB,
    _trail: &QueryTrail<'_, CurrentUd, Walked>,
) -> Result<Option<CurrentUd>, DbError> {
    Ok(db.get_current_ud()?.map(CurrentUd::from_current_du_db))
}

#[cfg(test)]
mod tests {
    use crate::db::BcDbRo;
    use crate::schema::queries::tests;
    use dubp_common_doc::BlockNumber;
    use durs_bc_db_reader::current_metadata::current_ud::CurrentUdDb;
    use serde_json::json;

    static mut DB_TEST_CURRENT_UD_1: Option<BcDbRo> = None;

    #[test]
    fn test_graphql_current_ud() {
        let mut mock_db = BcDbRo::new();

        // Define mock db expectations here

        mock_db.expect_get_current_ud().times(1).returning(|| {
            Ok(Some(CurrentUdDb {
                amount: 1_000,
                base: 0,
                block_number: BlockNumber(1),
                common_time: 1_488_987_127,
                members_count: 59,
                monetary_mass: 59_000,
            }))
        });

        let schema = tests::setup(mock_db, unsafe { &mut DB_TEST_CURRENT_UD_1 });

        tests::test_gql_query(
            schema,
            "{ currentUd { amount, base, blockNumber, commonTime, membersCount, monetaryMass } }",
            json!({
                "data": {
                    "currentUd": {
                        "amount": 1_000,
                        "base": 0,
                        "blockNumber": 1,
                        "commonTime": 1_488_987_127.0,
                        "membersCount": 59,
                        "monetaryMass": 59000
                    }
                }
            }),
        )
    }
}

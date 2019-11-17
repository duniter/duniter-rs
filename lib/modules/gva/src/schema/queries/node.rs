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

// ! Module execute GraphQl schema node query

use crate::context::Context;
use crate::schema::entities::node::{Node, Summary};
use juniper::Executor;
use juniper::FieldResult;
use juniper_from_schema::{QueryTrail, Walked};

pub(crate) fn execute(
    executor: &Executor<'_, Context>,
    _trail: &QueryTrail<'_, Node, Walked>,
) -> FieldResult<Node> {
    Ok(Node {
        summary: Summary {
            software: executor.context().get_software_name(),
            version: executor.context().get_software_version(),
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::db::MockBcDbTrait;
    use crate::schema::queries::tests;
    use serde_json::json;

    #[test]
    fn test_graphql_current() {
        let schema = tests::setup(MockBcDbTrait::new());

        tests::test_gql_query(
            schema,
            "{ node { summary { software, version } } }",
            json!({
                "data": {
                    "node": {
                        "summary": {
                            "software": "soft_name",
                            "version": "soft_version"
                        }
                    }
                }
            }),
        )
    }
}

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

// ! Module execute GraphQl schema queries

pub mod block;
pub mod blocks;
pub mod current;
pub mod node;

use durs_bc_db_reader::DbError;

pub(crate) fn db_err_to_juniper_err(e: DbError) -> juniper::FieldError {
    juniper::FieldError::from(format!("Db error: {:?}", e))
}

#[cfg(test)]
mod tests {

    use crate::context;
    use crate::db::MockBcDbTrait;
    use crate::graphql::graphql;
    use crate::schema::{create_schema, Schema};
    use actix_web::dev::Body;
    use actix_web::http;
    use actix_web::test;
    use actix_web::web;
    use juniper::http::GraphQLRequest;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;
    use std::sync::Arc;

    pub(crate) fn setup(mock_db: MockBcDbTrait) -> web::Data<Arc<Schema>> {
        context::init(mock_db, "soft_name", "soft_version");

        web::Data::new(std::sync::Arc::new(create_schema()))
    }

    pub(crate) fn test_gql_query(
        schema: web::Data<Arc<Schema>>,
        gql_query: &str,
        expected_response: serde_json::Value,
    ) {
        let resp = test::block_on(graphql(
            schema,
            web::Json(GraphQLRequest::new(gql_query.to_owned(), None, None)),
        ))
        .unwrap();
        assert_eq!(resp.status(), http::StatusCode::OK);
        if let Some(Body::Bytes(ref body_bytes)) = resp.body().as_ref() {
            assert_eq!(
                expected_response,
                serde_json::Value::from_str(
                    &String::from_utf8(body_bytes.to_vec())
                        .expect("response have invalid utf8 format.")
                )
                .expect("response have invalid JSON format.")
            )
        } else {
            panic!("Response must contain body in bytes format.")
        }
    }
}

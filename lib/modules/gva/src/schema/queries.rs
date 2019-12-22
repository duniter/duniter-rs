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
pub mod current_ud;
pub mod node;

#[cfg(test)]
mod tests {

    use crate::context::GlobalContext;
    use crate::db::BcDbRo;
    use crate::graphql::graphql;
    use crate::schema::create_schema;
    use actix_web::web;
    use assert_json_diff::assert_json_eq;
    use juniper::http::GraphQLRequest;
    use std::sync::Arc;

    pub(crate) fn setup(
        mock_db: BcDbRo,
        db_container: &'static mut Option<BcDbRo>,
    ) -> web::Data<Arc<GlobalContext>> {
        // Give a static lifetime to the DB
        let db = durs_common_tools::fns::r#static::to_static_ref(mock_db, db_container);

        // Init global context
        web::Data::new(std::sync::Arc::new(GlobalContext::new(
            db,
            create_schema(),
            "soft_name",
            "soft_version",
        )))
    }

    pub(crate) fn test_gql_query(
        global_context: web::Data<Arc<GlobalContext>>,
        gql_query: &str,
        expected_response: serde_json::Value,
    ) {
        let resp = actix_rt::Runtime::new()
            .expect("fail to start async executor")
            .block_on(graphql(
                global_context,
                web::Json(GraphQLRequest::new(gql_query.to_owned(), None, None)),
            ))
            .expect("async executor crashed");
        assert_json_eq!(expected_response, resp.0)
    }
}

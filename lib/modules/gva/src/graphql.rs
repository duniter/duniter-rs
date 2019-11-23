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
// web server implementaion based on actix-web

//! Module that execute graphql queries

use crate::context::{GlobalContext, QueryContext};
use actix_web::{web, Error, HttpResponse};
use futures::future::Future;
use juniper::http::GraphQLRequest;
use std::sync::Arc;

pub(crate) fn graphql(
    global_context: web::Data<Arc<GlobalContext>>,
    data: web::Json<GraphQLRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let query_context = QueryContext::from(global_context.as_ref());
    web::block(move || {
        let result = data.execute(&global_context.schema, &query_context);
        serde_json::to_string(&result)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

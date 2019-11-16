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

use crate::context;
use crate::schema::{create_schema, Schema};
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use durs_common_tools::fatal_error;
use durs_conf::DuRsConf;
use durs_module::SoftwareMetaDatas;
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::net::SocketAddr;
use std::sync::Arc;

fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:3000/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn graphql(
    schema: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let context = crate::context::get_context();
    web::block(move || {
        let result = data.execute(&schema, context);
        serde_json::to_string(&result)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}

pub fn start_web_server(soft_meta_datas: &SoftwareMetaDatas<DuRsConf>) -> std::io::Result<()> {
    info!("GVA web server start.");
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    // Create Juniper schema
    let schema = std::sync::Arc::new(create_schema());

    // Instanciate the context
    let db_path = durs_conf::get_blockchain_db_path(soft_meta_datas.profile_path.clone());
    if let Ok(db) = durs_bc_db_reader::open_db_ro(&std::path::Path::new(&db_path)) {
        context::init(db);
    } else {
        fatal_error!("GVA: fail to open DB.");
    };

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind(addr)?
    .run()
}

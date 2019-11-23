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

use crate::context::GlobalContext;
use crate::db::BcDbRo;
use crate::graphql::graphql;
use crate::schema::create_schema;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
#[cfg(not(test))]
use durs_common_tools::fatal_error;
use durs_conf::DuRsConf;
use durs_module::SoftwareMetaDatas;
use durs_network_documents::host::Host;
use durs_network_documents::url::Url;
use juniper::http::graphiql::graphiql_source;
use std::net::SocketAddr;

/// Database readonly handler (access to database)
static mut DB_RO_HANDLER: Option<BcDbRo> = None;

fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub fn start_web_server(
    soft_meta_datas: &SoftwareMetaDatas<DuRsConf>,
    host: Host,
    port: u16,
) -> std::io::Result<()> {
    info!("GVA web server start...");

    // Define listen addrs
    let addrs: Vec<SocketAddr> =
        Url::from_host_port_path(host, port, None).to_listenable_addr("http")?;

    // Get DB
    #[cfg(not(test))]
    let db = {
        let db_path = durs_conf::get_blockchain_db_path(soft_meta_datas.profile_path.clone());
        if let Ok(db) = durs_bc_db_reader::open_db_ro(&std::path::Path::new(&db_path)) {
            db
        } else {
            fatal_error!("GVA: fail to open DB.");
        }
    };
    #[cfg(test)]
    let db = BcDbRo::new();

    // Give a static lifetime to the DB
    let db = durs_common_tools::fns::r#static::to_static_ref(db, unsafe { &mut DB_RO_HANDLER });

    // Create global context
    let global_context = std::sync::Arc::new(GlobalContext::new(
        db,
        create_schema(),
        soft_meta_datas.soft_name,
        soft_meta_datas.soft_version,
    ));

    // Start http server
    HttpServer::new(move || {
        App::new()
            .data(global_context.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/graphql").route(web::post().to_async(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
    })
    .bind(&addrs[..])?
    .run()
}

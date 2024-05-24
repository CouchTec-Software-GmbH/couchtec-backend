mod db;
mod handlers;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use db::CouchDB;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = String::from("https://couchdb-app-service.azurewebsites.net/projects");
    let db_username = String::from("admin");
    let db_password = String::from("8RzuxhQ7");

    let couchdb = Arc::new(CouchDB::new(db_url, db_username, db_password));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(couchdb.clone()))
            .route("/projects/{id}", web::get().to(handlers::get_document))
            .route("/projects/{id}", web::put().to(handlers::put_document))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

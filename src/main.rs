mod db;
mod handlers;
mod auth;

use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use db::CouchDB;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = String::from("https://couchdb-app-service.azurewebsites.net");
    let db_username = String::from("admin");
    let db_password = String::from("8RzuxhQ7");

    let couchdb = Arc::new(CouchDB::new(db_url, db_username, db_password));
    let user_manager = Arc::new(Mutex::new(auth::UserManager::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(couchdb.clone()))
            .app_data(web::Data::new(user_manager.clone()))
            .route("/projects/{id}", web::get().to(handlers::get_document))
            .route("/projects/{id}", web::put().to(handlers::put_document))
            .route("/login", web::post().to(handlers::login))
            .route("/register", web::post().to(handlers::register))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

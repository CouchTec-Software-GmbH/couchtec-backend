mod db;
mod handlers;
mod auth;
mod email;

use actix_web::{web, App, HttpServer, http::header};
use actix_cors::Cors;
use email::EmailManager;
use std::sync::{Arc, Mutex};
use db::CouchDB;
use auth::UserManager;
use std::env;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

        
    let db_url = env::var("DB_URL").expect("DB URL must be set (e.g: https://couchdb-app-service.azurewebsites.net)");
    let db_username = env::var("DB_USERNAME").expect("DB Username must be set");
    let db_password = env::var("DB_PASSWORD").expect("DB Password must be set");

    let smtp_email = env::var("SMTP_EMAIL").expect("SMTP_EMAIL must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let couchdb = Arc::new(CouchDB::new(db_url, db_username, db_password));
    let user_manager = Arc::new(Mutex::new(UserManager::new()));
    let email_manager = match EmailManager::new(&smtp_email, &smtp_password) {
        Ok(manager) => Arc::new(manager),
        Err(e) => {
            eprintln!("Failed to create EmailManager: {:?}", e);
            std::process::exit(1);
        }
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);


        App::new()
            .wrap(cors)
            .app_data(web::Data::new(couchdb.clone()))
            .app_data(web::Data::new(user_manager.clone()))
            .app_data(web::Data::new(email_manager.clone()))
            .route("/{id}", web::get().to(handlers::get_document))
            .route("/{id}", web::put().to(handlers::put_document))
            .route("/login", web::post().to(handlers::login))
            .route("/register", web::post().to(handlers::register))
            .route("/pre-register", web::post().to(handlers::pre_register))
            .route("/uuids/{id}", web::get().to(handlers::get_uuids))
            .route("/uuids/{id}", web::post().to(handlers::post_uuid))
            .route("/uuids/{id}/{uuid}", web::delete().to(handlers::delete_uuid))
            .route("/pre-reset", web::post().to(handlers::send_reset_email))
            .route("/reset", web::post().to(handlers::reset_password))
            .route("/user/{id}", web::delete().to(handlers::delete_user))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}

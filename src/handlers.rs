use actix_web::{web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::db::CouchDB;
use serde_json::Value;
use crate::auth::UserManager;
use serde::Deserialize;

pub async fn get_document(db: web::Data<Arc<CouchDB>>, id: web::Path<String>) -> impl Responder {
    match db.get_document(&id).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::NotFound().body(format!("Document with id {} not found", id))
        }
    }
}

pub async fn put_document(db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<Value>) -> impl Responder {
    match db.put_document(&id, data.into_inner()).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

#[derive(Deserialize)]
pub struct AuthData {
    username: String,
    password: String,
}

pub async fn login(auth_data: web::Json<AuthData>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let user_manager = user_manager.lock().unwrap();
    match user_manager.sign_in(auth_data.username.clone(), auth_data.password.clone()) {
        Ok(session_id) => HttpResponse::Ok().json(format!("Login successful. Session ID: {}", session_id)),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::Unauthorized().body(format!("Login failed: {}", e))
        }
    }
}

pub async fn register(auth_data: web::Json<AuthData>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = user_manager.lock().unwrap();
    user_manager.register(auth_data.username.clone(), auth_data.password.clone());
    HttpResponse::Ok().body("User registered")
}

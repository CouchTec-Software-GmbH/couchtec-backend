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
    email: String,
    password: String,
}

pub async fn login(db: web::Data<Arc<CouchDB>>, auth_data: web::Json<AuthData>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = user_manager.lock().unwrap();
#[derive(Deserialize)]
pub struct ResetPasswordData {
    uuid: String,
    password: String,
}


    let user_data = {
        let user = user_manager.get_user(&auth_data.email);
        if user.is_none() {
            let fetched_user = db.get_user(&auth_data.email).await.ok();
            if let Some(ref user) = fetched_user {
                user_manager.insert_user(user.clone());
            }
            fetched_user
        } else {
            user.map(|u| u.clone())
        }
    };
    if user_data.is_none() {
        return HttpResponse::NotFound().body("Login failed: User not found");
    }
    let user_data = user_data.unwrap();
    match user_manager.sign_in(auth_data.password.clone(), user_data) {
        Ok(session_id) => HttpResponse::Ok().json(format!("{}", session_id)),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::Unauthorized().body(format!("Login failed: {}", e))
        }
    }
}

pub async fn register(db: web::Data<Arc<CouchDB>>, auth_data: web::Json<AuthData>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = user_manager.lock().unwrap();
    if user_manager.user_exists(&auth_data.email) || db.get_user(&auth_data.email).await.is_ok() {
        return HttpResponse::Conflict().body("User already exists");
    }
    let user = user_manager.register(auth_data.email.clone(), auth_data.password.clone());
    match db.put_user(user).await {
        Ok(_) => HttpResponse::Ok().json("User registered successfully"),
        Err(e) => {
            user_manager.remove_user(&auth_data.email);
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

pub async fn get_uuids(db: web::Data<Arc<CouchDB>>, id: web::Path<String>) -> impl Responder {
    match db.get_user(&id).await {
        Ok(user) => HttpResponse::Ok().json(user.uuids),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::NotFound().body(format!("User with email {} not found", id))
        }
    }
}

pub async fn put_uuids(db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<Vec<String>>) -> impl Responder {
    let mut user = match db.get_user(&id).await {
        Ok(user) => user,
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::NotFound().body(format!("User with email {} not found", id));
        }
    };
    user.uuids = data.into_inner();
    match db.put_user(user).await {
        Ok(_) => HttpResponse::Ok().json("UUIDs updated successfully"),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}
pub async fn reset_password(data: web::Json<ResetPasswordData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error")
    };
       
    // Does code exist?
    if let Some(email) = user_manager.get_email_from_code(&data.uuid) {
        // Does User exist?
        if !(user_manager.user_exists(&email) || db.get_user(&email).await.is_ok()) {
            return HttpResponse::Conflict().body("User does not exists");
        }
        let newsletter = match db.get_user(&email).await {
            Ok(user) => user.newsletter,
            Err(_) => return HttpResponse::NotFound().body("User not found"),
        };
        // Get new user && insert into cache
        let user = user_manager.change_password(&email, &data.password, newsletter);
        // Put new user into db
        return match db.put_user(user.clone()).await {
            Ok(_) => HttpResponse::Ok().body("Password changed successfully"),
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error {:?}", e);
                HttpResponse::InternalServerError().body("Internal Server Error")
            }
        }
    }
    HttpResponse::NotFound().body("No email found for this uuid")
}

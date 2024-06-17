use actix_web::{web, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::db::CouchDB;
use serde_json::Value;
use crate::auth::UserManager;
use crate::email::EmailManager;
use serde::Deserialize;

pub async fn get_document(db: web::Data<Arc<CouchDB>>, id: web::Path<String>) -> impl Responder {
    println!("Get_document handler");
    match db.get_document_data(&id).await {
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
pub struct LoginData {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct PreRegisterData {
    email: String,
    password: String,
    newsletter: bool,
}

#[derive(Deserialize)]
pub struct RegisterData{
    uuid: String,
}

#[derive(Deserialize)]
pub struct PreResetData {
    email: String,
}

#[derive(Deserialize)]
pub struct ResetData {
    uuid: String,
    password: String,
}

#[derive(Deserialize)]
pub struct AddUuid {
    uuid: String
}


pub async fn login(db: web::Data<Arc<CouchDB>>, auth_data: web::Json<LoginData>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error")
    };
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
    let user_data = match user_data {
        Some(data) => data,
        None => return HttpResponse::NotFound().body("Login failed: User not found")
    };
    match user_manager.sign_in(auth_data.password.clone(), user_data) {
        Ok(session_id) => HttpResponse::Ok().json(format!("{}", session_id)),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::Unauthorized().body(format!("Login failed: {}", e))
        }
    }
}

pub async fn register(auth_data: web::Json<RegisterData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    println!("Registered");
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error")
    };
    if let Ok(user) = user_manager.register(auth_data.uuid.clone()) {
        match db.put_user(user.clone()).await {
            Ok(_) => return HttpResponse::Ok().json("User registered successfully"),
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error: {:?}", e);
                return HttpResponse::InternalServerError().body("Internal Server Error");
            }
        }
    }
    HttpResponse::NotFound().body(format!("Internal Server Error"))
}

pub async fn pre_register(auth_data: web::Json<PreRegisterData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>, email_manager: web::Data<Arc<EmailManager>>) -> impl Responder {
    println!("Pre register");
    let url = "http://localhost";
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) =>  return HttpResponse::InternalServerError().body("Internal Server Error")
    };
    if user_manager.user_exists(&auth_data.email) || db.get_user(&auth_data.email).await.is_ok() {
        return HttpResponse::Conflict().body("User already exists");
    }

    let user_uuid = user_manager.pre_register(auth_data.email.clone(), auth_data.password.clone(), auth_data.newsletter.clone());
    let subject = "Activate Account";
    let body = format!("Click this link to activate your account: {}/auth?activate={}", url, user_uuid);
    println!("{}", &auth_data.email);
    match email_manager.send_email(&auth_data.email, subject, &body) {
        Ok(_) => {
            return HttpResponse::Ok().body("Activate email sent successfully");
        },
        Err(_) => {
            return HttpResponse::InternalServerError().body("Internal Server Error");
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

pub async fn post_uuid(db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<AddUuid>) -> impl Responder {
    let mut user = match db.get_user(&id).await {
        Ok(user) => user,
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::NotFound().body(format!("User with email {} not found", id));
        }
    };
    user.uuids.push(data.uuid.clone());
    match db.put_user(user).await {
        Ok(_) => HttpResponse::Ok().json("UUIDs updated successfully"),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

pub async fn delete_uuid(db: web::Data<Arc<CouchDB>>, path: web::Path<(String, String)>) -> impl Responder {
    let (id, uuid) = path.into_inner();
    let mut user = match db.get_user(&id).await {
        Ok(user) => user,
        Err(e) => {
            println!("Error: {:?}", e);
            return HttpResponse::NotFound().body(format!("User with email {} not found", id));
        }
    };
    user.uuids.retain(|x| !x.eq(&uuid.as_str()));
    match db.put_user(user).await {
        Ok(_) => HttpResponse::Ok().json("UUIDs updated successfully"),
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

pub async fn send_reset_email(data: web::Json<PreResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>, email_manager: web::Data<Arc<EmailManager>> ) -> impl Responder {
    let url = "http://localhost";
    println!("Reset email request for: {}", data.email);
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error")
    };
    if !(user_manager.user_exists(&data.email) || db.get_user(&data.email).await.is_ok()) {
        return HttpResponse::Conflict().body("User does not exists");
    }
    let onetimepassword = user_manager.insert_reset_email_code(data.email.clone());
    println!("Reset code: {}", onetimepassword);
    let subject = "Password Reset";
    let body = format!("Click this link to reset your password: {}/auth?code={}", url, onetimepassword);
    match email_manager.send_email(&data.email, subject, &body) {
        Ok(_) => {
            println!("Reset email sent successfully");
            HttpResponse::Ok().body("Reset email sent successfully")
        },
        Err(e) => {
            println!("Error: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

pub async fn reset_password(data: web::Json<ResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
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

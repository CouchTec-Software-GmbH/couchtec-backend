use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use crate::db::CouchDB;
use crate::utils::{self, ApiResponse};
use crate::AppConfig;
use serde_json::Value;
use crate::auth::UserManager;
use crate::email::EmailManager;
use serde::Deserialize;

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

pub async fn pre_register(auth_data: web::Json<PreRegisterData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>, email_manager: web::Data<Arc<EmailManager>>, app_config: web::Data<AppConfig>) -> impl Responder {
    let url = &app_config.url;
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) =>  return ApiResponse::InternalServerError.to_response()
    };
    if user_manager.user_exists(&auth_data.email) || db.get_user(&auth_data.email).await.is_ok() {
        return ApiResponse::Conflict.to_response()
    }

    let user_uuid = user_manager.pre_register(auth_data.email.clone(), auth_data.password.clone(), auth_data.newsletter.clone());
    let subject = "Activate Account";
    let body = format!("Click this link to activate your account: {}/auth?activate={}", url, user_uuid);
    match email_manager.send_email(&auth_data.email, subject, &body) {
        Ok(_) => {
            ApiResponse::Ok.to_response()
        },
        Err(_) => {
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn register(auth_data: web::Json<RegisterData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };
    if let Ok(user) = user_manager.register(auth_data.uuid.clone()) {
        match db.put_user(user.clone()).await {
            Ok(_) => return ApiResponse::Ok.to_response(),
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error: {:?}", e);
                return ApiResponse::InternalServerError.to_response()
            }
        }
    }
    ApiResponse::NotFound.to_response()
}

pub async fn login(auth_data: web::Json<LoginData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
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
        None => return ApiResponse::NotFound.to_response()
    };
    match user_manager.login(auth_data.password.clone(), user_data) {
        Ok(session_id) => HttpResponse::Ok().json(session_id.to_string()),
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::Unauthorized.to_response()
        }
    }
}

pub async fn logout(user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(guard) => guard,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };
    
    // Verify Session Token
    let token_id = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    user_manager.logout(token_id);
    ApiResponse::Ok.to_response()
}

pub async fn send_reset_email(data: web::Json<PreResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>, email_manager: web::Data<Arc<EmailManager>>, app_config: web::Data<AppConfig>) -> impl Responder {
    let url = &app_config.url;
    println!("Sending Reset email request for: {}", data.email);
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };
    if !(user_manager.user_exists(&data.email) || db.get_user(&data.email).await.is_ok()) {
        return ApiResponse::NotFound.to_response()
    }
    let onetimepassword = user_manager.insert_reset_email_code(data.email.clone());
    println!("Reset code: {}", onetimepassword);
    let subject = "Password Zurücksetzung";
    let body = format!("Klicken Sie diesen Link um Ihr Password zurückzusetzen: {}/auth?code={}", url, onetimepassword);
    match email_manager.send_email(&data.email, subject, &body) {
        Ok(_) => {
            println!("Reset email sent successfully");
            ApiResponse::Ok.to_response()
        },
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn reset_password(data: web::Json<ResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };
       
    // Does code exist?
    if let Some(email) = user_manager.get_email_from_code(&data.uuid) {
        // Does User exist?
        if !(user_manager.user_exists(&email) || db.get_user(&email).await.is_ok()) {
            return ApiResponse::NotFound.to_response()
        }
        let newsletter = match db.get_user(&email).await {
            Ok(user) => user.newsletter,
            Err(_) => return ApiResponse::NotFound.to_response()
        };
        // Get new user && insert into cache
        let user = user_manager.change_password(&email, &data.password, newsletter);
        // Put new user into db
        return match db.put_user(user.clone()).await {
            Ok(_) => ApiResponse::Ok.to_response(),
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error {:?}", e);
                ApiResponse::InternalServerError.to_response()
            }
        }
    }
    ApiResponse::NotFound.to_response()
}

pub async fn get_document(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    match db.get_document_data(&id).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::NotFound.to_response()
        }
    }
}

pub async fn put_document(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  data: web::Json<Value>, req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    // Put document
    match db.put_document(&id, data.into_inner()).await {
        Ok(doc) => HttpResponse::Ok().json(doc),
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::InternalServerError.to_response()
        }
    }
}


pub async fn get_uuids(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    match db.get_user(&id).await {
        Ok(user) => HttpResponse::Ok().json(user.uuids),
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::NotFound.to_response()
        }
    }
}

pub async fn post_uuid(user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest, db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<AddUuid>) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    let mut user = match db.get_user(&id).await {
        Ok(user) => user,
        Err(e) => {
            println!("Error: {:?}", e);
            return ApiResponse::NotFound.to_response()
        }
    };
    user.uuids.push(data.uuid.clone());
    match db.put_user(user).await {
        Ok(_) => HttpResponse::Ok().json("UUIDs updated successfully"),
        Err(e) => {
            println!("Error: {:?}", e);
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn delete_uuid(path: web::Path<(String, String)>, user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

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
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn delete_user(req: HttpRequest, email: web::Path<String> , user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return ApiResponse::InternalServerError.to_response()
    };

    // Verify Session Token
    let token_id = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    let user_db_deleted = match db.delete_user(&email.as_str()).await {
        Ok(_) => true,
        Err(_) => false
    };
    if !user_db_deleted {
        return ApiResponse::InternalServerError.to_response();
    }
    user_manager.logout(token_id);
    user_manager.delete_user(email.as_str());

    HttpResponse::Ok().body("User deleted successfully")
}

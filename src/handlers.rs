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
        Err(_) =>  {
            println!("pre-register: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
    if user_manager.user_exists(&auth_data.email) || db.get_user(&auth_data.email).await.is_ok() {
        return ApiResponse::Conflict.to_response()
    }

    let user_uuid = user_manager.pre_register(auth_data.email.clone(), auth_data.password.clone(), auth_data.newsletter.clone());
    let subject = "Activate Account";
    let body = format!("Click this link to activate your account: {}/auth?activate={}", url, user_uuid);
    match email_manager.send_email(&auth_data.email, subject, &body) {
        Ok(_) => {
            println!("pre-register: OK");
            ApiResponse::Ok.to_response()
        },
        Err(_) => {
            println!("pre-register: 500 (email)");
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn register(auth_data: web::Json<RegisterData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) =>  {
            println!("register: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
    if let Ok(user) = user_manager.register(auth_data.uuid.clone()) {
        match db.put_user(user.clone()).await {
            Ok(_) => {
                println!("register: OK");
                return ApiResponse::Ok.to_response();
            },
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error: {:?}", e);
                println!("register: 500 (put_user)");
                return ApiResponse::InternalServerError.to_response()
            }
        }
    }
    ApiResponse::NotFound.to_response()
}

pub async fn login(auth_data: web::Json<LoginData>, db: web::Data<Arc<CouchDB>>, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) =>  {
            println!("login: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
    let user_data = {
        let user = user_manager.get_user(&auth_data.email);
        if user.is_none() {
            println!("login: user not found in cache");

            let fetched_user = match db.get_user(&auth_data.email).await {
                Ok(user) => user,
                Err(e) => {
                    println!("login: user not found in db: {:?}", e);
                    return ApiResponse::InternalServerError.to_response();
                }
            };
            user_manager.insert_user(fetched_user.clone());
            // let fetched_user = db.get_user(&auth_data.email).await.ok();
            // if let Some(ref user) = fetched_user {
            //     println!("login: user found in db");
            //     user_manager.insert_user(user.clone());
            // }
            Some(fetched_user)
        } else {
            user.map(|u| u.clone())
        }
    };
    let user_data = match user_data {
        Some(data) => data,
        None => {
            println!("login: 404 (get_user cache & db)");
            return ApiResponse::NotFound.to_response()
        }
    };
    match user_manager.login(auth_data.password.clone(), user_data) {
        Ok(session_id) => {
            println!("login: OK");
            HttpResponse::Ok().json(session_id.to_string())
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("login: 401 (username & password don't match)");
            ApiResponse::Unauthorized.to_response()
        }
    }
}

pub async fn logout(user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(user_manager) => user_manager,
        Err(_) =>  {
            println!("logout: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
    
    // Verify Session Token
    let token_id = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => {
            println!("logout: invalid session token");
            return e.to_response();
        }
    };

    user_manager.logout(token_id);
    println!("logout: OK");
    ApiResponse::Ok.to_response()
}

pub async fn send_reset_email(data: web::Json<PreResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>, email_manager: web::Data<Arc<EmailManager>>, app_config: web::Data<AppConfig>) -> impl Responder {
    let url = &app_config.url;
    println!("Sending Reset email request for: {}", data.email);
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) =>  {
            println!("send_reset_email: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
    if !(user_manager.user_exists(&data.email) || db.get_user(&data.email).await.is_ok()) {
        println!("send_reset_email: 404 (user_exists & db.get_user)");
        return ApiResponse::NotFound.to_response()
    }
    let onetimepassword = user_manager.insert_reset_email_code(data.email.clone());
    println!("Reset code: {}", onetimepassword);
    let subject = "Password Zurücksetzung";
    let body = format!("Klicken Sie diesen Link um Ihr Password zurückzusetzen: {}/auth?code={}", url, onetimepassword);
    match email_manager.send_email(&data.email, subject, &body) {
        Ok(_) => {
            println!("send_reset_email: OK");
            ApiResponse::Ok.to_response()
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("send_reset_email: 500 (send_email)");
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn reset_password(data: web::Json<ResetData>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("reset_password: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response()
        }
    };
       
    // Does code exist?
    if let Some(email) = user_manager.get_email_from_code(&data.uuid) {
        // Does User exist?
        if !(user_manager.user_exists(&email) || db.get_user(&email).await.is_ok()) {
            println!("reset_password: 404 (user_exists & db.get_user)");
            return ApiResponse::NotFound.to_response()
        }
        let newsletter = match db.get_user(&email).await {
            Ok(user) => user.newsletter,
            Err(_) => {
                println!("reset_password: 404 (db.get_user)");
                return ApiResponse::NotFound.to_response();
            }
        };
        // Get new user && insert into cache
        let user = user_manager.change_password(&email, &data.password, newsletter);
        // Put new user into db
        return match db.put_user(user.clone()).await {
            Ok(_) => {
                println!("reset_password: OK");
                ApiResponse::Ok.to_response()
            },
            Err(e) => {
                user_manager.remove_user(&user.email);
                println!("Error {:?}", e);
                println!("reset_password: 500 (db.put_user)");
                ApiResponse::InternalServerError.to_response()
            }
        }
    }
    println!("reset_password: 404 (get_email_from_code)");
    ApiResponse::NotFound.to_response()
}

pub async fn get_document(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("get_document: 500 (user_manager)");
            return ApiResponse::InternalServerError.to_response();
        }
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    match db.get_document_data(&id).await {
        Ok(doc) => {
            println!("get_document: OK");
            HttpResponse::Ok().json(doc)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("get_document: 404 (get_document_data)");
            ApiResponse::NotFound.to_response()
        }
    }
}

pub async fn get_config(user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  req: HttpRequest) -> impl Responder {
    // let user_manager = match user_manager.lock() {
    //     Ok(manager) => manager,
    //     Err(_) => return ApiResponse::InternalServerError.to_response()
    // };

    // Verify Session Token
    // let _ = match utils::verfiy_session_token(&req, &user_manager) {
    //     Ok(token) => token,
    //     Err(e) => return e.to_response(),
    // };

    match db.get_config_data().await {
        Ok(doc) => {
            println!("get_config: OK");
            HttpResponse::Ok().json(doc)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("get_config: 404 (db.get_config_data)");
            ApiResponse::NotFound.to_response()
        }
    }
}

pub async fn put_document(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  data: web::Json<Value>, req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("put_document: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    // Put document
    match db.put_document(&id, data.into_inner()).await {
        Ok(doc) => {
            println!("put_document: OK");
            HttpResponse::Ok().json(doc)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("put_document: 500 db.put_document");
            ApiResponse::InternalServerError.to_response()
        }
    }
}


pub async fn get_uuids(id: web::Path<String>, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>,  req: HttpRequest) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("get_uuids: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
    };

    // Verify Session Token
    let _ = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    match db.get_user(&id).await {
        Ok(user) => {
            println!("get_uuids: OK");
            HttpResponse::Ok().json(user.uuids)
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("get_uuids: 404 db.get_user");
            ApiResponse::NotFound.to_response()
        }
    }
}

pub async fn post_uuid(user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest, db: web::Data<Arc<CouchDB>>, id: web::Path<String>, data: web::Json<AddUuid>) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("post_uuid: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
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
            println!("post_uuid: 404 db.get_user");
            return ApiResponse::NotFound.to_response()
        }
    };
    user.uuids.push(data.uuid.clone());
    match db.put_user(user).await {
        Ok(_) => {
            println!("post_uuid: OK");
            HttpResponse::Ok().json("UUIDs updated successfully")
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("post_uuid: 500 db.put_user");
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn delete_uuid(path: web::Path<(String, String)>, user_manager: web::Data<Arc<Mutex<UserManager>>>, req: HttpRequest, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("delete_uuid: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
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
            println!("delete_uuid: 404 db.get_user");
            return HttpResponse::NotFound().body(format!("User with email {} not found", id));
        }
    };
    user.uuids.retain(|x| !x.eq(&uuid.as_str()));
    match db.put_user(user).await {
        Ok(_) => {
            println!("delete_uuid: OK");
            HttpResponse::Ok().json("UUIDs updated successfully")
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("delete_uuid: 500 db.put_user");
            ApiResponse::InternalServerError.to_response()
        }
    }
}

pub async fn delete_user(req: HttpRequest, email: web::Path<String> , user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let mut user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("delete_user: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
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
        println!("delete_user: 500 db.delete_user");
        return ApiResponse::InternalServerError.to_response();
    }
    user_manager.logout(token_id);
    user_manager.delete_user(email.as_str());

    println!("delete_user: OK");
    HttpResponse::Ok().body("User deleted successfully")
}

pub async fn get_last_uuid(req: HttpRequest, user_manager: web::Data<Arc<Mutex<UserManager>>>, db: web::Data<Arc<CouchDB>>) -> impl Responder {
    let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => {
            println!("get_last_uuid: 500 user_manager");
            return ApiResponse::InternalServerError.to_response();
        }
    };

    let token = match utils::verfiy_session_token(&req, &user_manager) {
        Ok(token) => token,
        Err(e) => return e.to_response(),
    };

    println!("Verified token: {}", &token);
    
    let email = match user_manager.get_email_from_token(&token) {
        Some(email) => email,
        None => {
            println!("get_last_uuid: 404 get_email_from_token");
            return ApiResponse::NotFound.to_response();
        }
    };

    println!("Got email: {}", &email);

    let user = match db.get_user(&email).await {
        Ok(user) => user,
        Err(_) => {
            println!("get_last_uuid: 404 db.get_user");
            return ApiResponse::NotFound.to_response();
        }
    };

    println!("Got user with email: {}", user.email);
    println!("Got user with last_uuid: {}", user.email);

    println!("get_last_uuid: OK");
    HttpResponse::Ok().body(user.last_uuid)
}

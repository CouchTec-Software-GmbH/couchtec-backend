use actix_web::{web, HttpRequest, HttpResponse};
use crate::UserManager;
use std::sync::{Arc, Mutex };

pub fn extract_session_token(req: &HttpRequest) -> Option<String> {
    req.headers().get("Authorization")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_str| {
            if header_str.starts_with("Bearer ") {
                Some(header_str[7..].to_string())
            } else {
                None 
            }
        })
}

pub struct RequestContext {
    pub session_token: String,
    pub email: String,
}

pub fn get_request_context(req: HttpRequest, user_manager: web::Data<Arc<Mutex<UserManager>>>) -> Result<RequestContext, HttpResponse> {
    let session_token = match extract_session_token(&req) {
        Some(token) => token, 
        None => return Err(HttpResponse::Unauthorized().body("No session token found")),
    };
     let user_manager = match user_manager.lock() {
        Ok(manager) => manager,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Internal Server Error"))
    };

    let email = match user_manager.get_email_from_session_token(session_token.clone()) {
        Some(email) => email,
        None => return Err(HttpResponse::Unauthorized().body("Session token invalid")),
    };
    Ok(RequestContext {
        session_token,
        email,
    })
}

use actix_web::{HttpRequest, HttpResponse };
use crate::auth::UserManager;

pub enum ApiResponse {
    Ok,
    NotFound,
    Conflict,
    Unauthorized,
    InternalServerError,
}

impl ApiResponse {
    pub fn to_response(self) -> HttpResponse {
        match self {
            ApiResponse::Ok => HttpResponse::Ok().body("Ok"),
            ApiResponse::NotFound => HttpResponse::NotFound().body("Not found"),
            ApiResponse::Conflict => HttpResponse::NotFound().body("Conflict"),
            ApiResponse::Unauthorized => HttpResponse::Unauthorized().body("Unauthorized"),
            ApiResponse::InternalServerError => HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

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

pub fn verfiy_session_token(req: &HttpRequest, user_manager: &UserManager) -> Result<String, ApiResponse> {
    let token_id = extract_session_token(req).ok_or(ApiResponse::Unauthorized)?;
    if !user_manager.session_token_valid(token_id.clone()) {
        return Err(ApiResponse::Unauthorized);
    }
    Ok(token_id)
}

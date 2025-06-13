use actix_web::HttpRequest;
use crate::services::auth_service::{AuthService, Claims};

pub fn get_user_id_from_request(req: &HttpRequest) -> Result<i32, String> {
    let token = extract_token_from_request(req)?;
    let claims = AuthService::verify_token(&token)?;
    Ok(claims.sub)
}

pub fn extract_token_from_request(req: &HttpRequest) -> Result<String, String> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or("Authorization header missing")?
        .to_str()
        .map_err(|_| "Invalid authorization header")?;

    if auth_header.starts_with("Bearer ") {
        Ok(auth_header[7..].to_string())
    } else {
        Err("Invalid authorization format".to_string())
    }
}

pub fn extract_optional_user_id(req: &HttpRequest) -> Option<i32> {
    get_user_id_from_request(req).ok()
}

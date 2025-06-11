use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

pub fn get_user_id_from_request(req: &HttpRequest) -> Option<i32> {
    let auth_header = req.headers().get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;

    if !auth_str.starts_with("Bearer ") {
        return None;
    }

    let token = &auth_str[7..]; // Remove "Bearer " prefix

    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;

    Some(token_data.claims.sub)
}

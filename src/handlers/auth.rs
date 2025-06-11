use actix_web::{web, HttpResponse, Responder, Result};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::{LoginRequest, NewUser, RegisterRequest, User};
use crate::schema::users;
use crate::utils::database::get_connection;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: User,
}

pub async fn register(req: web::Json<RegisterRequest>) -> Result<impl Responder> {
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    // Check if user already exists
    let existing_user = users::table
        .filter(users::name.eq(&req.name))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Database error"))?;

    if existing_user.is_some() {
        return Ok(HttpResponse::BadRequest().json("User already exists"));
    }

    let new_user = NewUser {
        name: req.name.clone(),
        social_id: req.social_id.clone(),
        avatar_url: req.avatar_url.clone(),
        region: req.region.clone(),
        gender: req.gender.clone(),
        profile: None,
    };

    let user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to create user"))?;

    let token = generate_token(user.id)?;

    Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
}

pub async fn login(req: web::Json<LoginRequest>) -> Result<impl Responder> {
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let user = users::table
        .filter(users::name.eq(&req.name))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Database error"))?;

    match user {
        Some(user) => {
            let token = generate_token(user.id)?;
            Ok(HttpResponse::Ok().json(AuthResponse { token, user }))
        }
        None => Ok(HttpResponse::Unauthorized().json("Invalid credentials")),
    }
}

pub async fn logout() -> Result<impl Responder> {
    // For JWT tokens, logout is handled client-side by removing the token
    Ok(HttpResponse::Ok().json("Logged out successfully"))
}

fn generate_token(user_id: i32) -> Result<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| HttpResponse::InternalServerError().json("Token generation failed"))
}

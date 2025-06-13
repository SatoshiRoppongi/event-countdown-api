use actix_web::{web, HttpResponse, Responder, Result};
use oauth2::{AuthorizationCode, TokenResponse};
use serde::Deserialize;

use crate::models::user::{LoginRequest, RegisterRequest, OAuthCallbackRequest};
use crate::services::auth_service::AuthService;
use crate::services::oauth_service::{OAuthService, GoogleUserInfo, TwitterUserInfo};
use crate::utils::database::get_connection;

pub async fn register(req: web::Json<RegisterRequest>) -> Result<impl Responder> {
    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    match AuthService::register_user(&mut conn, req.into_inner()) {
        Ok(auth_response) => Ok(HttpResponse::Ok().json(auth_response)),
        Err(error) => Ok(HttpResponse::BadRequest().json(error)),
    }
}

pub async fn login(req: web::Json<LoginRequest>) -> Result<impl Responder> {
    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    match AuthService::login_user(&mut conn, req.into_inner()) {
        Ok(auth_response) => Ok(HttpResponse::Ok().json(auth_response)),
        Err(error) => Ok(HttpResponse::Unauthorized().json(error)),
    }
}

pub async fn logout() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json("ログアウトしました"))
}

// Google OAuth endpoints
pub async fn google_login() -> Result<impl Responder> {
    match OAuthService::get_google_config() {
        Ok(config) => {
            match OAuthService::create_google_client(&config) {
                Ok(client) => {
                    let (auth_url, _csrf_token) = OAuthService::get_google_auth_url(&client);
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "auth_url": auth_url.to_string()
                    })))
                }
                Err(error) => Ok(HttpResponse::InternalServerError().json(error)),
            }
        }
        Err(error) => Ok(HttpResponse::InternalServerError().json(error)),
    }
}

pub async fn google_callback(req: web::Json<OAuthCallbackRequest>) -> Result<impl Responder> {
    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    let config = match OAuthService::get_google_config() {
        Ok(config) => config,
        Err(error) => return Ok(HttpResponse::InternalServerError().json(error)),
    };

    let client = match OAuthService::create_google_client(&config) {
        Ok(client) => client,
        Err(error) => return Ok(HttpResponse::InternalServerError().json(error)),
    };

    // Exchange authorization code for access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(req.code.clone()))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    let token = match token_result {
        Ok(token) => token,
        Err(error) => return Ok(HttpResponse::BadRequest().json(format!("Token exchange failed: {}", error))),
    };

    // Get user information from Google
    let user_info: GoogleUserInfo = match OAuthService::get_google_user_info(token.access_token().secret()).await {
        Ok(info) => info,
        Err(error) => return Ok(HttpResponse::BadRequest().json(error)),
    };

    // Create or update user
    match AuthService::create_oauth_user(
        &mut conn,
        user_info.name,
        Some(user_info.email),
        "google".to_string(),
        user_info.id,
        user_info.picture,
    ) {
        Ok(auth_response) => Ok(HttpResponse::Ok().json(auth_response)),
        Err(error) => Ok(HttpResponse::InternalServerError().json(error)),
    }
}

// Twitter OAuth endpoints (similar structure)
pub async fn twitter_login() -> Result<impl Responder> {
    // Twitter OAuth implementation would go here
    Ok(HttpResponse::NotImplemented().json("Twitter login not yet implemented"))
}

pub async fn twitter_callback(req: web::Json<OAuthCallbackRequest>) -> Result<impl Responder> {
    // Twitter OAuth callback implementation would go here
    Ok(HttpResponse::NotImplemented().json("Twitter callback not yet implemented"))
}

use crate::models::user::{NewUser, User, LoginRequest, RegisterRequest};
use crate::schema::users;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use diesel::PgConnection;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use chrono::{Duration, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub name: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
    pub gender: Option<String>,
    pub profile: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
            region: user.region,
            gender: user.gender,
            profile: user.profile,
        }
    }
}

pub struct AuthService;

impl AuthService {
    pub fn register_user(
        conn: &mut PgConnection,
        register_req: RegisterRequest,
    ) -> Result<AuthResponse, String> {
        if register_req.email.is_some() && register_req.password.is_none() {
            return Err("Password is required".to_string());
        }

        if let Some(ref email) = register_req.email {
            let existing_user = users::table
                .filter(users::email.eq(email))
                .first::<User>(conn)
                .optional()
                .map_err(|e| format!("Database error: {}", e))?;

            if existing_user.is_some() {
                return Err("Email already exists".to_string());
            }
        }

        let hashed_password = if let Some(password) = &register_req.password {
            Some(hash(password, DEFAULT_COST).map_err(|e| format!("Password hashing error: {}", e))?)
        } else {
            None
        };

        let new_user = NewUser {
            name: register_req.name,
            email: register_req.email,
            password: hashed_password,
            social_id: register_req.social_id,
            avatar_url: register_req.avatar_url,
            region: register_req.region,
            gender: register_req.gender,
            profile: None,
            oauth_provider: None,
            oauth_id: None,
        };

        let user: User = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)
            .map_err(|e| format!("Failed to create user: {}", e))?;

        let token = Self::generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub fn login_user(
        conn: &mut PgConnection,
        login_req: LoginRequest,
    ) -> Result<AuthResponse, String> {
        let user = if let Some(email) = &login_req.email {
            users::table
                .filter(users::email.eq(email))
                .first::<User>(conn)
                .optional()
                .map_err(|e| format!("Database error: {}", e))?
        } else if let Some(name) = &login_req.name {
            users::table
                .filter(users::name.eq(name))
                .first::<User>(conn)
                .optional()
                .map_err(|e| format!("Database error: {}", e))?
        } else {
            return Err("Email or name required".to_string());
        };

        let user = user.ok_or("User not found")?;

        if let Some(password) = &login_req.password {
            if let Some(ref stored_password) = user.password {
                if !verify(password, stored_password)
                    .map_err(|e| format!("Password verification error: {}", e))?
                {
                    return Err("Invalid password".to_string());
                }
            } else {
                return Err("This user does not support password authentication".to_string());
            }
        } else {
            return Err("Password required".to_string());
        }

        let token = Self::generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub fn create_oauth_user(
        conn: &mut PgConnection,
        name: String,
        email: Option<String>,
        oauth_provider: String,
        oauth_id: String,
        avatar_url: Option<String>,
    ) -> Result<AuthResponse, String> {
        let existing_user = users::table
            .filter(users::oauth_provider.eq(&oauth_provider))
            .filter(users::oauth_id.eq(&oauth_id))
            .first::<User>(conn)
            .optional()
            .map_err(|e| format!("Database error: {}", e))?;

        let user = if let Some(user) = existing_user {
            diesel::update(users::table.find(user.id))
                .set((
                    users::name.eq(&name),
                    users::email.eq(&email),
                    users::avatar_url.eq(&avatar_url),
                ))
                .get_result::<User>(conn)
                .map_err(|e| format!("Failed to update user: {}", e))?
        } else {
            let new_user = NewUser {
                name,
                email,
                password: None,
                social_id: None,
                avatar_url,
                region: None,
                gender: None,
                profile: None,
                oauth_provider: Some(oauth_provider),
                oauth_id: Some(oauth_id),
            };

            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result(conn)
                .map_err(|e| format!("Failed to create user: {}", e))?
        };

        let token = Self::generate_token(&user)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub fn generate_token(user: &User) -> Result<String, String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
        let now = Utc::now();
        let exp = now + Duration::hours(24);

        let claims = Claims {
            sub: user.id,
            name: user.name.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| format!("Token generation error: {}", e))
    }

    pub fn verify_token(token: &str) -> Result<Claims, String> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Token verification error: {}", e))
    }

    pub fn get_user_by_id(conn: &mut PgConnection, user_id: i32) -> Result<User, String> {
        users::table
            .find(user_id)
            .first::<User>(conn)
            .map_err(|e| format!("User not found: {}", e))
    }
}
use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub social_id: Option<String>,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
    pub gender: Option<String>,
    pub profile: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub social_id: Option<String>,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
    pub gender: Option<String>,
    pub profile: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
    pub gender: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: Option<String>,
    pub password: Option<String>,
    pub social_id: Option<String>,
    pub avatar_url: Option<String>,
    pub region: Option<String>,
    pub gender: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub social_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: Option<String>,
}

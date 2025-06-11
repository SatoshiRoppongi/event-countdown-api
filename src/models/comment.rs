use crate::schema::comments;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(crate::models::User, foreign_key = user_id))]
#[diesel(belongs_to(crate::models::Event, foreign_key = event_id))]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub user_id: Option<i32>,
    pub event_id: Option<i32>,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = comments)]
pub struct NewComment {
    pub user_id: Option<i32>,
    pub event_id: Option<i32>,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct CommentWithUser {
    pub id: i32,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub user_avatar: Option<String>,
    pub event_id: Option<i32>,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub event_id: i32,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ReportCommentRequest {
    pub reason: String,
}

use crate::schema::reports;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = reports)]
pub struct Report {
    pub id: i32,
    pub reporter_id: Option<i32>,
    pub target_comment_id: Option<i32>,
    pub reason: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = reports)]
pub struct NewReport {
    pub reporter_id: Option<i32>,
    pub target_comment_id: Option<i32>,
    pub reason: String,
}

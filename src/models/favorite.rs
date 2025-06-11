use crate::schema::favorites;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = favorites)]
#[diesel(primary_key(user_id, event_id))]
pub struct Favorite {
    pub user_id: i32,
    pub event_id: i32,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = favorites)]
pub struct NewFavorite {
    pub user_id: i32,
    pub event_id: i32,
}

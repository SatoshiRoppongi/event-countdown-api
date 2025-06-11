use crate::schema::events;
use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = events)]
pub struct Event {
    pub id: i32,
    pub event_type: Option<String>,
    pub name: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub source_type: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = events)]
pub struct NewEvent {
    pub event_type: Option<String>,
    pub name: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub source_type: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = events)]
pub struct UpdateEvent {
    pub event_type: Option<String>,
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EventWithTags {
    #[serde(flatten)]
    pub event: Event,
    pub tags: Vec<String>,
    pub is_favorited: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub event_type: Option<String>,
    pub location: Option<String>,
    pub tags: Option<String>,
    pub search: Option<String>,
}

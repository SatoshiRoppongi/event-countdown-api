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
    pub start_date_from: Option<NaiveDate>,
    pub start_date_to: Option<NaiveDate>,
    pub source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserEventRequest {
    pub name: String,
    pub event_type: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl UserEventRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("イベント名は必須です".to_string());
        }

        if self.name.len() > 255 {
            return Err("イベント名は255文字以下で入力してください".to_string());
        }

        if let Some(ref description) = self.description {
            if description.len() > 5000 {
                return Err("説明は5000文字以下で入力してください".to_string());
            }
        }

        if let Some(ref url) = self.url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("URLは http:// または https:// で始まる必要があります".to_string());
            }
        }

        if let Some(start_date) = self.start_date {
            if let Some(end_date) = self.end_date {
                if start_date > end_date {
                    return Err("終了日は開始日以降で設定してください".to_string());
                }
            }
        }

        Ok(())
    }

    pub fn to_new_event(&self) -> NewEvent {
        NewEvent {
            event_type: self.event_type.clone(),
            name: self.name.clone(),
            start_date: self.start_date,
            end_date: self.end_date,
            description: self.description.clone(),
            location: self.location.clone(),
            source_type: Some("user".to_string()),
            url: self.url.clone(),
            image_url: self.image_url.clone(),
        }
    }
}

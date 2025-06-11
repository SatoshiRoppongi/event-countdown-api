use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::models::NewEvent;

#[derive(Debug, Deserialize)]
pub struct ExternalEventResponse {
    pub events: Vec<ExternalEvent>,
}

#[derive(Debug, Deserialize)]
pub struct ExternalEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub category: Option<String>,
}

pub struct ExternalApiService {
    client: Client,
}

impl ExternalApiService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// 例: Connpassからイベントを取得
    pub async fn fetch_connpass_events(&self) -> Result<Vec<NewEvent>, Box<dyn Error>> {
        let url = "https://connpass.com/api/v1/event/?count=100";

        let response = self.client.get(url).send().await?;

        let json: serde_json::Value = response.json().await?;
        let mut events = Vec::new();

        if let Some(events_array) = json["events"].as_array() {
            for event_data in events_array {
                if let Ok(event) = self.parse_connpass_event(event_data) {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    fn parse_connpass_event(&self, data: &serde_json::Value) -> Result<NewEvent, Box<dyn Error>> {
        let title = data["title"].as_str().unwrap_or("").to_string();
        let description = data["description"].as_str().map(|s| s.to_string());
        let started_at = data["started_at"].as_str().unwrap_or("");
        let ended_at = data["ended_at"].as_str();
        let address = data["address"].as_str().map(|s| s.to_string());
        let event_url = data["event_url"].as_str().map(|s| s.to_string());

        // 日付をパース
        let start_date = if !started_at.is_empty() {
            chrono::DateTime::parse_from_rfc3339(started_at)
                .map(|dt| dt.naive_local().date())
                .ok()
        } else {
            None
        };

        let end_date = if let Some(ended_at) = ended_at {
            chrono::DateTime::parse_from_rfc3339(ended_at)
                .map(|dt| dt.naive_local().date())
                .ok()
        } else {
            None
        };

        Ok(NewEvent {
            event_type: Some("tech".to_string()),
            name: title,
            start_date,
            end_date,
            description,
            location: address,
            source_type: Some("connpass".to_string()),
            url: event_url,
            image_url: None,
        })
    }

    /// 外部APIからイベントを定期的に取得してDBに保存
    pub async fn sync_external_events(&self) -> Result<(), Box<dyn Error>> {
        let events = self.fetch_connpass_events().await?;

        // DBに保存するロジックをここに実装
        // 重複チェックも必要

        log::info!("Synced {} events from external APIs", events.len());
        Ok(())
    }
}

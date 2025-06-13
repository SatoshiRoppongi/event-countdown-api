use chrono::{NaiveDate, DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::models::{NewEvent, Event};
use crate::schema::events;

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

    /// Doorkeeper APIからイベントを取得
    pub async fn fetch_doorkeeper_events(&self) -> Result<Vec<NewEvent>, Box<dyn Error>> {
        let url = "https://api.doorkeeper.jp/events";
        
        let response = self.client.get(url).send().await?;
        let json: serde_json::Value = response.json().await?;
        let mut events = Vec::new();

        if let Some(events_array) = json.as_array() {
            for event_data in events_array {
                if let Ok(event) = self.parse_doorkeeper_event(event_data) {
                    events.push(event);
                }
            }
        }

        Ok(events)
    }

    fn parse_doorkeeper_event(&self, data: &serde_json::Value) -> Result<NewEvent, Box<dyn Error>> {
        let title = data["event"]["title"].as_str().unwrap_or("").to_string();
        let description = data["event"]["description"].as_str().map(|s| s.to_string());
        let starts_at = data["event"]["starts_at"].as_str().unwrap_or("");
        let ends_at = data["event"]["ends_at"].as_str();
        let venue_name = data["event"]["venue_name"].as_str().map(|s| s.to_string());
        let public_url = data["event"]["public_url"].as_str().map(|s| s.to_string());

        let start_date = if !starts_at.is_empty() {
            DateTime::parse_from_rfc3339(starts_at)
                .map(|dt| dt.naive_local().date())
                .ok()
        } else {
            None
        };

        let end_date = if let Some(ends_at) = ends_at {
            DateTime::parse_from_rfc3339(ends_at)
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
            location: venue_name,
            source_type: Some("doorkeeper".to_string()),
            url: public_url,
            image_url: None,
        })
    }

    /// Peatix APIからイベントを取得 (例)
    pub async fn fetch_peatix_events(&self) -> Result<Vec<NewEvent>, Box<dyn Error>> {
        // Peatix APIの実装
        // 実際のAPIエンドポイントとレスポンス形式に合わせて実装
        Ok(Vec::new())
    }

    /// 外部APIからイベントを定期的に取得してDBに保存
    pub async fn sync_external_events(&self, conn: &mut PgConnection) -> Result<usize, Box<dyn Error>> {
        let mut all_events = Vec::new();
        
        // 複数のAPIから取得
        let connpass_events = self.fetch_connpass_events().await?;
        all_events.extend(connpass_events);

        // Doorkeeperも取得 (エラーがあっても続行)
        if let Ok(doorkeeper_events) = self.fetch_doorkeeper_events().await {
            all_events.extend(doorkeeper_events);
        }

        let mut inserted_count = 0;

        // DBに保存（重複チェック付き）
        for event in all_events {
            if let Err(e) = self.save_event_if_not_exists(conn, &event) {
                log::warn!("Failed to save event: {:?}", e);
            } else {
                inserted_count += 1;
            }
        }

        log::info!("Synced {} new events from external APIs", inserted_count);
        Ok(inserted_count)
    }

    /// イベントが存在しない場合のみ保存
    fn save_event_if_not_exists(&self, conn: &mut PgConnection, new_event: &NewEvent) -> Result<(), Box<dyn Error>> {
        // URLと名前で重複チェック
        let existing = if let Some(ref url) = new_event.url {
            events::table
                .filter(events::url.eq(url))
                .first::<Event>(conn)
                .optional()?
        } else {
            events::table
                .filter(events::name.eq(&new_event.name))
                .filter(events::start_date.eq(new_event.start_date))
                .first::<Event>(conn)
                .optional()?
        };

        if existing.is_none() {
            diesel::insert_into(events::table)
                .values(new_event)
                .execute(conn)?;
        }

        Ok(())
    }

    /// イベント情報を外部APIで補完
    pub async fn enrich_event_data(&self, event: &mut NewEvent) -> Result<(), Box<dyn Error>> {
        // 位置情報APIを使用して座標を取得
        if let Some(ref location) = event.location {
            if let Ok(coordinates) = self.geocode_location(location).await {
                log::info!("Found coordinates for {}: {:?}", location, coordinates);
                // 座標情報をイベントに追加（必要に応じてスキーマ拡張）
            }
        }

        // 画像URLが無い場合、関連画像を取得
        if event.image_url.is_none() {
            if let Ok(image_url) = self.find_event_image(&event.name).await {
                event.image_url = Some(image_url);
            }
        }

        Ok(())
    }

    async fn geocode_location(&self, location: &str) -> Result<(f64, f64), Box<dyn Error>> {
        // Google Maps API や OpenStreetMap API を使用
        // 例: OpenStreetMap Nominatim API
        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
            urlencoding::encode(location)
        );

        let response = self.client.get(&url).send().await?;
        let json: serde_json::Value = response.json().await?;

        if let Some(first_result) = json.as_array().and_then(|arr| arr.first()) {
            let lat = first_result["lat"].as_str().and_then(|s| s.parse::<f64>().ok());
            let lon = first_result["lon"].as_str().and_then(|s| s.parse::<f64>().ok());

            if let (Some(lat), Some(lon)) = (lat, lon) {
                return Ok((lat, lon));
            }
        }

        Err("Failed to geocode location".into())
    }

    async fn find_event_image(&self, event_name: &str) -> Result<String, Box<dyn Error>> {
        // 画像検索APIを使用して関連画像を取得
        // 例: Unsplash API, Pixabay API など
        Err("Image search not implemented".into())
    }
}

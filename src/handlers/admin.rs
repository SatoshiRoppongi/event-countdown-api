use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use serde_json::json;

use crate::middleware::auth::get_user_id_from_request;
use crate::services::external_api::ExternalApiService;
use crate::utils::database::get_connection;

pub async fn sync_external_events(req: HttpRequest) -> Result<impl Responder> {
    // 管理者認証チェック（簡易版）
    let _user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Admin authentication required")),
    };

    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    let api_service = ExternalApiService::new();
    
    match api_service.sync_external_events(&mut conn).await {
        Ok(count) => Ok(HttpResponse::Ok().json(json!({
            "message": "External events synced successfully",
            "synced_count": count
        }))),
        Err(e) => {
            log::error!("Failed to sync external events: {:?}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to sync external events"))
        }
    }
}

pub async fn get_sync_status() -> Result<impl Responder> {
    // 同期状況の確認エンドポイント
    Ok(HttpResponse::Ok().json(json!({
        "last_sync": "2025-06-13T04:00:00Z", // 実際は最後の同期時刻を取得
        "sources": [
            {
                "name": "connpass",
                "status": "active",
                "last_success": "2025-06-13T04:00:00Z"
            },
            {
                "name": "doorkeeper", 
                "status": "active",
                "last_success": "2025-06-13T04:00:00Z"
            }
        ]
    })))
}
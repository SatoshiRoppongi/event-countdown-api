use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenvy::dotenv;
use std::env;

mod handlers;
mod middleware;
mod models;
mod schema;
mod services;
mod utils;

use handlers::{admin, auth, comments, events, users};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    log::info!("Starting server on port {}", port);

    HttpServer::new(|| {
        App::new().wrap(Logger::default()).service(
            web::scope("/api/v1")
                // 認証関連
                .service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/logout", web::post().to(auth::logout))
                        .route("/google", web::get().to(auth::google_login))
                        .route("/google/callback", web::post().to(auth::google_callback))
                        .route("/twitter", web::get().to(auth::twitter_login))
                        .route("/twitter/callback", web::post().to(auth::twitter_callback)),
                )
                // イベント関連
                .service(
                    web::scope("/events")
                        .route("", web::get().to(events::get_events))
                        .route("", web::post().to(events::create_event))
                        .route("/{id}", web::get().to(events::get_event))
                        .route("/{id}", web::put().to(events::update_event))
                        .route("/{id}", web::delete().to(events::delete_event))
                        .route("/{id}/comments", web::get().to(events::get_event_comments))
                        .route("/{id}/favorite", web::post().to(events::add_favorite))
                        .route("/{id}/favorite", web::delete().to(events::remove_favorite)),
                )
                // コメント関連
                .service(
                    web::scope("/comments")
                        .route("", web::post().to(comments::create_comment))
                        .route("/{id}", web::delete().to(comments::delete_comment))
                        .route("/{id}/report", web::post().to(comments::report_comment)),
                )
                // ユーザー関連
                .service(
                    web::scope("/users")
                        .route("/me", web::get().to(users::get_profile))
                        .route("/me", web::put().to(users::update_profile))
                        .route("/me/favorites", web::get().to(users::get_favorites)),
                )
                // 管理者関連
                .service(
                    web::scope("/admin")
                        .route("/sync/external-events", web::post().to(admin::sync_external_events))
                        .route("/sync/status", web::get().to(admin::get_sync_status)),
                ),
        )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

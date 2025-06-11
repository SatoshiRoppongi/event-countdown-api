use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use diesel::prelude::*;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{EventWithTags, UpdateUser, User};
use crate::schema::{event_tags, events, favorites, tags, users};
use crate::utils::database::get_connection;

pub async fn get_profile(req: HttpRequest) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let user = users::table
        .find(user_id)
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Database error"))?;

    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

pub async fn update_profile(
    req: HttpRequest,
    user_data: web::Json<UpdateUser>,
) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let updated_user = diesel::update(users::table.find(user_id))
        .set(user_data.into_inner())
        .get_result::<User>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to update user"))?;

    match updated_user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Ok(HttpResponse::NotFound().json("User not found")),
    }
}

pub async fn get_favorites(req: HttpRequest) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let favorite_events = favorites::table
        .inner_join(events::table)
        .filter(favorites::user_id.eq(user_id))
        .select(events::all_columns)
        .load::<crate::models::Event>(&mut conn)
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to fetch favorites"))?;

    // タグ情報を取得
    let mut events_with_tags = Vec::new();
    for event in favorite_events {
        let tags_data = event_tags::table
            .inner_join(tags::table)
            .filter(event_tags::event_id.eq(event.id))
            .select(tags::name)
            .load::<String>(&mut conn)
            .unwrap_or_default();

        events_with_tags.push(EventWithTags {
            event,
            tags: tags_data,
            is_favorited: Some(true),
        });
    }

    Ok(HttpResponse::Ok().json(events_with_tags))
}

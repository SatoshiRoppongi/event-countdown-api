use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use diesel::prelude::*;
use serde::Serialize;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{Event, EventQuery, EventWithTags, NewEvent, UpdateEvent, UserEventRequest};
use crate::schema::{event_tags, events, favorites, tags};
use crate::utils::database::get_connection;

#[derive(Serialize)]
struct EventsResponse {
    events: Vec<EventWithTags>,
    total: i64,
    page: i64,
    limit: i64,
}

pub async fn get_events(query: web::Query<EventQuery>, req: HttpRequest) -> Result<impl Responder> {
    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let user_id = get_user_id_from_request(&req).ok();

    let mut query_builder = events::table.into_boxed();

    // フィルタリング
    if let Some(ref event_type) = query.event_type {
        query_builder = query_builder.filter(events::event_type.eq(event_type));
    }

    if let Some(ref location) = query.location {
        query_builder = query_builder.filter(events::location.ilike(format!("%{}%", location)));
    }

    if let Some(ref source_type) = query.source_type {
        query_builder = query_builder.filter(events::source_type.eq(source_type));
    }

    // 日付範囲でのフィルタリング
    if let Some(start_date_from) = query.start_date_from {
        query_builder = query_builder.filter(events::start_date.ge(start_date_from));
    }

    if let Some(start_date_to) = query.start_date_to {
        query_builder = query_builder.filter(events::start_date.le(start_date_to));
    }

    // タグでのフィルタリング
    if let Some(ref tag_names) = query.tags {
        let tag_list: Vec<&str> = tag_names.split(',').map(|s| s.trim()).collect();
        if !tag_list.is_empty() {
            query_builder = query_builder
                .inner_join(event_tags::table.inner_join(tags::table))
                .filter(tags::name.eq_any(tag_list))
                .group_by(events::id);
        }
    }

    if let Some(ref search) = query.search {
        query_builder = query_builder.filter(
            events::name
                .ilike(format!("%{}%", search))
                .or(events::description.ilike(format!("%{}%", search)))
                .or(events::location.ilike(format!("%{}%", search))),
        );
    }

    let total = match query_builder
        .count()
        .get_result::<i64>(&mut conn) {
        Ok(count) => count,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to count events")),
    };

    let events_data = match query_builder
        .order(events::start_date.desc())
        .limit(limit)
        .offset(offset)
        .load::<Event>(&mut conn) {
        Ok(events) => events,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to fetch events")),
    };

    // タグとお気に入り情報を取得
    let mut events_with_tags = Vec::new();
    for event in events_data {
        let tags_data = event_tags::table
            .inner_join(tags::table)
            .filter(event_tags::event_id.eq(event.id))
            .select(tags::name)
            .load::<String>(&mut conn)
            .unwrap_or_default();

        let is_favorited = if let Some(uid) = user_id {
            favorites::table
                .filter(favorites::user_id.eq(uid))
                .filter(favorites::event_id.eq(event.id))
                .first::<crate::models::Favorite>(&mut conn)
                .optional()
                .map(|f| f.is_some())
                .unwrap_or(Some(false))
        } else {
            None
        };

        events_with_tags.push(EventWithTags {
            event,
            tags: tags_data,
            is_favorited,
        });
    }

    Ok(HttpResponse::Ok().json(EventsResponse {
        events: events_with_tags,
        total,
        page,
        limit,
    }))
}

pub async fn get_event(path: web::Path<i32>, req: HttpRequest) -> Result<impl Responder> {
    let event_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let user_id = get_user_id_from_request(&req);

    let event = events::table
        .find(event_id)
        .first::<Event>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Database error"))?;

    match event {
        Some(event) => {
            let tags_data = event_tags::table
                .inner_join(tags::table)
                .filter(event_tags::event_id.eq(event.id))
                .select(tags::name)
                .load::<String>(&mut conn)
                .unwrap_or_default();

            let is_favorited = if let Some(uid) = user_id {
                favorites::table
                    .filter(favorites::user_id.eq(uid))
                    .filter(favorites::event_id.eq(event.id))
                    .first::<crate::models::Favorite>(&mut conn)
                    .optional()
                    .map(|f| f.is_some())
                    .unwrap_or(Some(false))
            } else {
                None
            };

            let event_with_tags = EventWithTags {
                event,
                tags: tags_data,
                is_favorited,
            };

            Ok(HttpResponse::Ok().json(event_with_tags))
        }
        None => Ok(HttpResponse::NotFound().json("Event not found")),
    }
}

pub async fn create_event(
    req: HttpRequest,
    event_data: web::Json<UserEventRequest>,
) -> Result<impl Responder> {
    let _user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("認証が必要です")),
    };

    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("データベース接続に失敗しました")),
    };

    let event_request = event_data.into_inner();

    // バリデーション
    if let Err(error) = event_request.validate() {
        return Ok(HttpResponse::BadRequest().json(error));
    }

    let new_event = event_request.to_new_event();

    // イベントを作成
    let event = match diesel::insert_into(events::table)
        .values(&new_event)
        .get_result::<Event>(&mut conn) {
        Ok(event) => event,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("イベントの作成に失敗しました")),
    };

    // タグがある場合は関連付け
    if let Some(ref tag_names) = event_request.tags {
        for tag_name in tag_names {
            if let Err(e) = create_or_associate_tag(&mut conn, event.id, tag_name) {
                log::warn!("Failed to associate tag '{}': {:?}", tag_name, e);
            }
        }
    }

    // タグ情報を含めてレスポンス
    let tags_data = match event_tags::table
        .inner_join(tags::table)
        .filter(event_tags::event_id.eq(event.id))
        .select(tags::name)
        .load::<String>(&mut conn) {
        Ok(tags) => tags,
        Err(_) => Vec::new(),
    };

    let event_with_tags = EventWithTags {
        event,
        tags: tags_data,
        is_favorited: Some(false),
    };

    Ok(HttpResponse::Created().json(event_with_tags))
}

fn create_or_associate_tag(conn: &mut PgConnection, event_id: i32, tag_name: &str) -> Result<(), diesel::result::Error> {
    use crate::models::tag::{NewTag, Tag};
    use crate::models::event_tag::NewEventTag;

    // タグが存在するかチェック
    let tag = tags::table
        .filter(tags::name.eq(tag_name))
        .first::<Tag>(conn)
        .optional()?;

    let tag_id = if let Some(tag) = tag {
        tag.id
    } else {
        // タグが存在しない場合は作成
        let new_tag = NewTag {
            name: tag_name.to_string(),
        };
        diesel::insert_into(tags::table)
            .values(&new_tag)
            .get_result::<Tag>(conn)?
            .id
    };

    // イベントとタグの関連付け
    let new_event_tag = NewEventTag {
        event_id,
        tag_id,
    };

    diesel::insert_into(event_tags::table)
        .values(&new_event_tag)
        .execute(conn)?;

    Ok(())
}

pub async fn update_event(
    req: HttpRequest,
    path: web::Path<i32>,
    event_data: web::Json<UpdateEvent>,
) -> Result<impl Responder> {
    let _user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let event_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let updated_event = diesel::update(events::table.find(event_id))
        .set(event_data.into_inner())
        .get_result::<Event>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to update event"))?;

    match updated_event {
        Some(event) => Ok(HttpResponse::Ok().json(event)),
        None => Ok(HttpResponse::NotFound().json("Event not found")),
    }
}

pub async fn delete_event(req: HttpRequest, path: web::Path<i32>) -> Result<impl Responder> {
    let _user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let event_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let deleted_count = diesel::delete(events::table.find(event_id))
        .execute(&mut conn)
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to delete event"))?;

    if deleted_count > 0 {
        Ok(HttpResponse::Ok().json("Event deleted successfully"))
    } else {
        Ok(HttpResponse::NotFound().json("Event not found"))
    }
}

pub async fn add_favorite(req: HttpRequest, path: web::Path<i32>) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let event_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let new_favorite = crate::models::NewFavorite { user_id, event_id };

    diesel::insert_into(favorites::table)
        .values(&new_favorite)
        .execute(&mut conn)
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to add favorite"))?;

    Ok(HttpResponse::Ok().json("Added to favorites"))
}

pub async fn remove_favorite(req: HttpRequest, path: web::Path<i32>) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let event_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    let deleted_count = diesel::delete(
        favorites::table
            .filter(favorites::user_id.eq(user_id))
            .filter(favorites::event_id.eq(event_id)),
    )
    .execute(&mut conn)
    .map_err(|_| HttpResponse::InternalServerError().json("Failed to remove favorite"))?;

    if deleted_count > 0 {
        Ok(HttpResponse::Ok().json("Removed from favorites"))
    } else {
        Ok(HttpResponse::NotFound().json("Favorite not found"))
    }
}

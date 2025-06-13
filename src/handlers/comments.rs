use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use diesel::prelude::*;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{
    Comment, CommentWithUser, CreateCommentRequest, NewComment, ReportCommentRequest,
};
use crate::schema::{comments, reports, users};
use crate::utils::database::get_connection;

pub async fn create_comment(
    req: HttpRequest,
    comment_data: web::Json<CreateCommentRequest>,
) -> Result<impl Responder> {
    let user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Authentication required")),
    };

    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    let new_comment = NewComment {
        user_id: Some(user_id),
        event_id: Some(comment_data.event_id),
        content: comment_data.content.clone(),
    };

    let comment = match diesel::insert_into(comments::table)
        .values(&new_comment)
        .get_result::<Comment>(&mut conn) {
        Ok(comment) => comment,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to create comment")),
    };

    // ユーザー情報と一緒に返す
    let comment_with_user = comments::table
        .left_join(users::table.on(comments::user_id.eq(users::id.nullable())))
        .filter(comments::id.eq(comment.id))
        .select((
            comments::id,
            comments::user_id,
            users::name.nullable(),
            users::avatar_url.nullable(),
            comments::event_id,
            comments::content,
            comments::created_at,
        ))
        .first::<(
            i32,
            Option<i32>,
            Option<String>,
            Option<String>,
            Option<i32>,
            String,
            Option<chrono::NaiveDateTime>,
        )>(&mut conn)
        .map(
            |(id, user_id, user_name, user_avatar, event_id, content, created_at)| {
                CommentWithUser {
                    id,
                    user_id,
                    user_name,
                    user_avatar,
                    event_id,
                    content,
                    created_at,
                }
            },
        )
        .map_err(|_| {
            HttpResponse::InternalServerError().json("Failed to fetch comment with user info")
        });

    let comment_with_user = match comment_with_user {
        Ok(comment) => comment,
        Err(response) => return Ok(response),
    };

    Ok(HttpResponse::Created().json(comment_with_user))
}

pub async fn delete_comment(req: HttpRequest, path: web::Path<i32>) -> Result<impl Responder> {
    let user_id = match get_user_id_from_request(&req) {
        Ok(id) => id,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Authentication required")),
    };

    let comment_id = path.into_inner();
    let mut conn = match get_connection() {
        Ok(conn) => conn,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database connection failed")),
    };

    // コメントの所有者チェック
    let comment = match comments::table
        .find(comment_id)
        .first::<Comment>(&mut conn)
        .optional() {
        Ok(comment) => comment,
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database error")),
    };

    match comment {
        Some(comment) => {
            if comment.user_id != Some(user_id) {
                return Ok(HttpResponse::Forbidden().json("You can only delete your own comments"));
            }

            let deleted_count = diesel::delete(comments::table.find(comment_id))
                .execute(&mut conn)
                .map_err(|_| {
                    HttpResponse::InternalServerError().json("Failed to delete comment")
                })?;

            if deleted_count > 0 {
                Ok(HttpResponse::Ok().json("Comment deleted successfully"))
            } else {
                Ok(HttpResponse::NotFound().json("Comment not found"))
            }
        }
        None => Ok(HttpResponse::NotFound().json("Comment not found")),
    }
}

pub async fn report_comment(
    req: HttpRequest,
    path: web::Path<i32>,
    report_data: web::Json<ReportCommentRequest>,
) -> Result<impl Responder> {
    let user_id = get_user_id_from_request(&req)
        .ok_or_else(|| HttpResponse::Unauthorized().json("Authentication required"))?;

    let comment_id = path.into_inner();
    let mut conn = get_connection()
        .map_err(|_| HttpResponse::InternalServerError().json("Database connection failed"))?;

    // コメントが存在するかチェック
    let comment_exists = comments::table
        .find(comment_id)
        .first::<Comment>(&mut conn)
        .optional()
        .map_err(|_| HttpResponse::InternalServerError().json("Database error"))?
        .is_some();

    if !comment_exists {
        return Ok(HttpResponse::NotFound().json("Comment not found"));
    }

    let new_report = crate::models::NewReport {
        reporter_id: Some(user_id),
        target_comment_id: Some(comment_id),
        reason: report_data.reason.clone(),
    };

    diesel::insert_into(reports::table)
        .values(&new_report)
        .execute(&mut conn)
        .map_err(|_| HttpResponse::InternalServerError().json("Failed to create report"))?;

    Ok(HttpResponse::Ok().json("Report submitted successfully"))
}

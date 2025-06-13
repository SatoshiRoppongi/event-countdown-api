// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        event_id -> Nullable<Int4>,
        content -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    event_tags (event_id, tag_id) {
        event_id -> Int4,
        tag_id -> Int4,
    }
}

diesel::table! {
    events (id) {
        id -> Int4,
        #[max_length = 50]
        event_type -> Nullable<Varchar>,
        #[max_length = 255]
        name -> Varchar,
        start_date -> Nullable<Date>,
        end_date -> Nullable<Date>,
        description -> Nullable<Text>,
        #[max_length = 255]
        location -> Nullable<Varchar>,
        #[max_length = 20]
        source_type -> Nullable<Varchar>,
        url -> Nullable<Text>,
        image_url -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    favorites (user_id, event_id) {
        user_id -> Int4,
        event_id -> Int4,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    reports (id) {
        id -> Int4,
        reporter_id -> Nullable<Int4>,
        target_comment_id -> Nullable<Int4>,
        reason -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tags (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 255]
        social_id -> Nullable<Varchar>,
        avatar_url -> Nullable<Text>,
        #[max_length = 100]
        region -> Nullable<Varchar>,
        #[max_length = 50]
        gender -> Nullable<Varchar>,
        profile -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        #[max_length = 50]
        oauth_provider -> Nullable<Varchar>,
        #[max_length = 255]
        oauth_id -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Nullable<Varchar>,
        #[max_length = 255]
        password -> Nullable<Varchar>,
    }
}

diesel::joinable!(comments -> events (event_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(event_tags -> events (event_id));
diesel::joinable!(event_tags -> tags (tag_id));
diesel::joinable!(favorites -> events (event_id));
diesel::joinable!(favorites -> users (user_id));
diesel::joinable!(reports -> comments (target_comment_id));
diesel::joinable!(reports -> users (reporter_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    event_tags,
    events,
    favorites,
    reports,
    tags,
    users,
);

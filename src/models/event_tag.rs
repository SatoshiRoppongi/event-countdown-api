use crate::schema::event_tags;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = event_tags)]
#[diesel(primary_key(event_id, tag_id))]
pub struct EventTag {
    pub event_id: i32,
    pub tag_id: i32,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = event_tags)]
pub struct NewEventTag {
    pub event_id: i32,
    pub tag_id: i32,
}
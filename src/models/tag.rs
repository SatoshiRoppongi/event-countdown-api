use crate::schema::tags;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag {
    pub name: String,
}

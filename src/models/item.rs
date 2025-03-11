use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Insertable};
use crate::schema::items;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = items)]
pub struct NewItem {
    pub name: String,
}

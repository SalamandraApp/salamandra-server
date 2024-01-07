use diesel::prelude::*;
use diesel::pg::PgConnection;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Serialize, Deserialize, Identifiable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub pfp_url: Option<String>,
    pub date_of_birth: Option<NaiveDateTime>,
    pub date_joined: NaiveDateTime,
    pub training_state: i32,
    pub fitness_level: i32,
    pub height: i32,
}


use diesel::prelude::*;
use uuid::Uuid;
use serde::Serialize;

#[derive(Queryable, Serialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub pfp_url: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub date_joined: chrono::DateTime<chrono::Utc>,
    pub training_state: i32,
    pub fitness_level: i32,
    pub height: Option<i32>,
}


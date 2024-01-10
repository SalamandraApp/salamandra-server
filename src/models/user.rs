use diesel::prelude::*;
use diesel::pg::sql_types::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    // pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub pfp_url: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub date_joined: Option<chrono::DateTime<chrono::Utc>>,
    pub training_state: Option<i32>,
    pub fitness_level: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct RegisteredUser {
    // pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
}

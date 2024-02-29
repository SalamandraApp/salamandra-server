use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Queryable, Serialize, Insertable, Clone)]
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

#[derive(Deserialize)]
pub struct UserSearchParams {
    pub username: String,
    /*
    Maybe
    pub display_name: String,
    pub location: String
    */
}

#[derive(Serialize)]
pub struct UserSearchResult {
    pub users: Vec<UserInfo>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub username: String,
    pub uuid: uuid::Uuid,
}

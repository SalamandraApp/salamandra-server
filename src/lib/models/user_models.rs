use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{NaiveDate, Utc};
use crate::schema::users;

#[derive(Queryable, Insertable, Selectable, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub date_joined: NaiveDate,
    pub date_of_birth: Option<NaiveDate>,
    pub height: Option<i32>,
    pub weight: Option<f32>,
    pub gender: Option<i32>,
    pub fitness_goal: Option<i32>,
    pub fitness_level: Option<i32>,
}
impl Default for User {
    fn default() -> Self {
        User {
            id: Uuid::new_v4(), 
            username: "username".to_string(),
            display_name: "display name".to_string(),
            date_joined: Utc::now().naive_utc().date(),
            date_of_birth: None,
            height: None,
            weight: None,
            gender: None,
            fitness_goal: None,
            fitness_level: None,
        }
    }
}

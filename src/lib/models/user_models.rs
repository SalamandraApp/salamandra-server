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
    pub height: Option<i16>,
    pub weight: Option<f32>,
    pub gender: Option<i16>,
    pub fitness_goal: Option<i16>,
    pub fitness_level: Option<i16>,
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

#[derive(Insertable, Serialize, Deserialize, Clone, Debug, PartialEq, AsChangeset)]
#[diesel(table_name = users)]
pub struct UncompleteUser {
    // USERNAME cannot be changed
    pub display_name: Option<String>,
    pub date_joined: Option<NaiveDate>,
    pub date_of_birth: Option<NaiveDate>,
    pub height: Option<i16>,
    pub weight: Option<f32>,
    pub gender: Option<i16>,
    pub fitness_goal: Option<i16>,
    pub fitness_level: Option<i16>,
} impl Default for UncompleteUser{
    fn default() -> Self {
        UncompleteUser {
            display_name: None,
            date_joined: None,
            date_of_birth: None,
            height: None,
            weight: None,
            gender: None,
            fitness_goal: None,
            fitness_level: None,
        }
    }
}

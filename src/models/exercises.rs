use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Serialize)]
#[diesel(table_name = crate::schema::exercises)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Exercise {
    pub id: i32,
    pub name: String,
}

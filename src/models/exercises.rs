use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::exercises)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Exercise {
    pub exercise_name: String,
}

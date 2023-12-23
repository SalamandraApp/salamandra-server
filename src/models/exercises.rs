use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::exercises)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Exercise {
    pub id: u64,
    pub name: String,
}

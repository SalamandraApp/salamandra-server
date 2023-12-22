use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::exercises)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Exercise {
    pub exercise_name: String,
}

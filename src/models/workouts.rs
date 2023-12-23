use diesel::prelude::*;
use crate::models::users::User;

#[derive(Queryable, Selectable, Insertable, Associations, Identifiable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::workouts)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Workout {
    pub id: u64,
    pub user_id: u64,
}

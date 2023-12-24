use diesel::prelude::*;
use crate::models::users::User;

#[derive(Associations, Identifiable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::workouts)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Workout {
    pub id: i32,
    pub user_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workouts)]
pub struct NewWorkout {
    pub user_id: i32,
}


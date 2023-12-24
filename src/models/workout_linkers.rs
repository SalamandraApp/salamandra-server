use diesel::prelude::*;
use crate::models::exercises::Exercise;
use crate::models::workouts::Workout;

#[derive(Queryable, Associations, Identifiable)]
#[diesel(belongs_to(Workout))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = crate::schema::workout_linkers)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct WorkoutLinker {
    pub id: u64,
    pub workout_id: u64,
    pub exercise_id: u64,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::workout_linkers)]
pub struct NewWorkoutLinker {
    pub workout_id: u64,
    pub exercise_id: u64,
}

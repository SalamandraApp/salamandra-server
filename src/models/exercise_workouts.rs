use diesel::prelude::*;
use crate::models::exercises::Exercise;
use crate::models::workouts::Workout;

#[derive(Queryable, Selectable, Insertable, Associations, Identifiable)]
#[diesel(belongs_to(Workout))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = crate::schema::exercise_workout)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ExerciseWorkout {
    pub id: u64,
    pub workout_id: u64,
    pub exercise_id: u64,
}

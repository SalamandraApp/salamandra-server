use diesel::prelude::*;
use crate::models::exercises::Exercise;
use crate::models::workouts::Workout;

#[derive(Queryable, Associations, Identifiable)]
#[diesel(belongs_to(Workout))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = crate::schema::workout_linkers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkoutLinker {
    pub id: i32,
    pub workout_id: i32,
    pub exercise_id: i32,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::workout_linkers)]
pub struct NewWorkoutLinker {
    pub workout_id: i32,
    pub exercise_id: i32,
}

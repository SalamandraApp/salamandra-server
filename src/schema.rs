// @generated automatically by Diesel CLI.

diesel::table! {
    exercises (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 64]
        username -> Varchar,
        #[max_length = 64]
        password -> Varchar,
    }
}

diesel::table! {
    workout_linkers (id) {
        id -> Int4,
        workout_id -> Int4,
        exercise_id -> Int4,
    }
}

diesel::table! {
    workouts (id) {
        id -> Int4,
        user_id -> Int4,
    }
}

diesel::joinable!(workout_linkers -> exercises (exercise_id));
diesel::joinable!(workout_linkers -> workouts (workout_id));
diesel::joinable!(workouts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    exercises,
    users,
    workout_linkers,
    workouts,
);

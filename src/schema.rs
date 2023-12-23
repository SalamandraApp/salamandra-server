// @generated automatically by Diesel CLI.

diesel::table! {
    exercise_workout (id) {
        id -> Unsigned<Bigint>,
        workout_id -> Unsigned<Bigint>,
        exercise_id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    exercises (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 32]
        name -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 32]
        username -> Varchar,
        #[max_length = 64]
        password -> Varchar,
    }
}

diesel::table! {
    workouts (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
    }
}

diesel::joinable!(exercise_workout -> exercises (exercise_id));
diesel::joinable!(exercise_workout -> workouts (workout_id));
diesel::joinable!(workouts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    exercise_workout,
    exercises,
    users,
    workouts,
);

// @generated automatically by Diesel CLI.

diesel::table! {
    exercises (exercise_name) {
        #[max_length = 32]
        exercise_name -> Varchar,
    }
}

diesel::table! {
    users (username) {
        #[max_length = 32]
        username -> Varchar,
        #[max_length = 64]
        password -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    exercises,
    users,
);

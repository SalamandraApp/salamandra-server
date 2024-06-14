// @generated automatically by Diesel CLI.

diesel::table! {
    exercises (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        main_muscle_group -> Nullable<Int4>,
        secondary_muscle_group -> Nullable<Int4>,
        necessary_equipment -> Nullable<Int4>,
        exercise_type -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        display_name -> Varchar,
        date_joined -> Date,
        date_of_birth -> Nullable<Date>,
        height -> Nullable<Int4>,
        weight -> Nullable<Float4>,
        gender -> Nullable<Int4>,
        fitness_goal -> Int4,
        fitness_level -> Int4,
    }
}

diesel::table! {
    wktemplateelements (id) {
        id -> Uuid,
        workout_template_id -> Uuid,
        exercise_id -> Uuid,
        position -> Int4,
        reps -> Int4,
        sets -> Int4,
        weight -> Int4,
        rest -> Int4,
        super_set -> Nullable<Int4>,
    }
}

diesel::table! {
    workouttemplates (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        date_created -> Date,
    }
}

diesel::joinable!(wktemplateelements -> exercises (exercise_id));
diesel::joinable!(wktemplateelements -> workouttemplates (workout_template_id));
diesel::joinable!(workouttemplates -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    exercises,
    users,
    wktemplateelements,
    workouttemplates,
);

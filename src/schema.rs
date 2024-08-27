// @generated automatically by Diesel CLI.

diesel::table! {
    exercises (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        main_muscle_group -> Nullable<Int2>,
        secondary_muscle_group -> Nullable<Int2>,
        necessary_equipment -> Nullable<Int2>,
        exercise_type -> Nullable<Int2>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        display_name -> Varchar,
        date_joined -> Date,
        date_of_birth -> Nullable<Date>,
        height -> Nullable<Int2>,
        weight -> Nullable<Float4>,
        gender -> Nullable<Int2>,
        fitness_goal -> Nullable<Int2>,
        fitness_level -> Nullable<Int2>,
    }
}

diesel::table! {
    wkexecutionelements (id) {
        id -> Uuid,
        workout_execution_id -> Uuid,
        exercise_id -> Uuid,
        position -> Int2,
        exercise_number -> Int2,
        reps -> Int2,
        set_number -> Int2,
        weight -> Nullable<Float4>,
        rest -> Int2,
        super_set -> Nullable<Int2>,
        time -> Int4,
    }
}

diesel::table! {
    wktemplateelements (id) {
        id -> Uuid,
        workout_template_id -> Uuid,
        exercise_id -> Uuid,
        position -> Int2,
        reps -> Int2,
        sets -> Int2,
        weight -> Nullable<Float4>,
        rest -> Int2,
        super_set -> Nullable<Int2>,
    }
}

diesel::table! {
    workoutexecutions (id) {
        id -> Uuid,
        workout_template_id -> Uuid,
        date -> Date,
        survey -> Int2,
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

diesel::joinable!(wkexecutionelements -> exercises (exercise_id));
diesel::joinable!(wkexecutionelements -> workoutexecutions (workout_execution_id));
diesel::joinable!(wktemplateelements -> exercises (exercise_id));
diesel::joinable!(wktemplateelements -> workouttemplates (workout_template_id));
diesel::joinable!(workoutexecutions -> workouttemplates (workout_template_id));
diesel::joinable!(workouttemplates -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    exercises,
    users,
    wkexecutionelements,
    wktemplateelements,
    workoutexecutions,
    workouttemplates,
);

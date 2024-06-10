// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        display_name -> Varchar,
        date_joined -> Timestamptz,
        date_of_birth -> Nullable<Date>,
        height -> Nullable<Int4>,
        weight -> Nullable<Float4>,
        gender -> Nullable<Int4>,
        fitness_goal -> Int4,
        fitness_level -> Int4,
    }
}

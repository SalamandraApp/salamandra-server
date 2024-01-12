// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        display_name -> Varchar,
        pfp_url -> Nullable<Text>,
        date_of_birth -> Nullable<Date>,
        date_joined -> Timestamptz,
        training_state -> Int4,
        fitness_level -> Int4,
        height -> Nullable<Int4>,
    }
}

// @generated automatically by Diesel CLI.

diesel::table! {
    users (username) {
        #[max_length = 32]
        username -> Varchar,
        #[max_length = 64]
        password -> Varchar,
    }
}

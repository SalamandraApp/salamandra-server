use actix_web::{web, Responder};
// use serde_json::json;
// use tokio::task;

// use diesel::prelude::*;
// use diesel::insert_into;

use crate::models::user::{RegisteredUser, User, AccessTokenClaims};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/{user_id}", web::get().to(get_user))
       .route("/{user_id}", web::post().to(add_user));
}

async fn get_user(_username: web::Path<String>) -> impl Responder {
    "Get user is not yet implemented"
}

async fn add_user(
    user_id: web::Path<String>, 
    register_info: web::Json<RegisteredUser>) -> impl Responder {

    // Check token
    // Create User struct
    let uuid = uuid::Uuid::parse_str(&user_id).expect("Invalid UUID format");
    let new_user = User {
        id: uuid,
        username: register_info.username.clone(),
        display_name: register_info.display_name.clone().unwrap_or_default(),        
        date_joined: chrono::Utc::now(),
        training_state: 0,
        fitness_level: 0,
        
        pfp_url: None,
        date_of_birth: None,
        height: None,
    };

    // Insert into db
    // Schedule task1

    "Add user is not yet implemented"
}

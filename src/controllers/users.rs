use actix_web::{web, Responder};
use serde::Deserialize;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_users))
       .route("/{username}", web::get().to(get_user))
       .route("/{username}/workouts", web::get().to(get_workout))
       .route("/{username}/workouts", web::post().to(add_workout));
}

#[derive(Deserialize)]
pub struct WorkoutPayload {
    exercise_ids: Vec<i32>,
}

async fn list_users() -> impl Responder {
    "List users is not yet implemented"
}

async fn get_user(username: web::Path<String>) -> impl Responder {
    "Get user is not yet implemented"
}

async fn get_workout(username: web::Path<String>) -> impl Responder {
    "Get workouts is not yet implemented"
}

async fn add_workout(username: web::Path<String>) -> impl Responder {
    "Add workout is not yet implemented"
}



use actix_web::{web, Responder};
use serde_json::json;
use serde::Deserialize;
use tokio::task;

use diesel::prelude::*;
use diesel::insert_into;

use crate::models::user::{RegisteredUser, User};


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/{user_id}", web::get().to(get_user))
       .route("/{user_id}", web::post().to(add_user));
}

async fn get_user(_username: web::Path<String>) -> impl Responder {
    "Get user is not yet implemented"
}


async fn add_user(_username: web::Path<String>) -> impl Responder {
    "Add user is not yet implemented"
}

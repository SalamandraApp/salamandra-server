use actix_web::{web, post, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[post("/login")]
async fn login(info: web::Json<LoginRequest>) -> HttpResponse {
    // Login logic here
    HttpResponse::Ok().body("Login successful")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
}

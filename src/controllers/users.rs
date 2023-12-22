use actix_web::{web, Responder, HttpResponse};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_users))
       .route("/{id}", web::get().to(get_user));
}

async fn list_users() -> impl Responder {
    "List users is not implemented"
}

async fn get_user() -> impl Responder {
    "Get user is not implemented"
}



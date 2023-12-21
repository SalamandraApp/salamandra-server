use actix_web::{web, HttpResponse};


#[derive(serde::Deserialize)]
pub struct UserInfo {
    id: String,
    password: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_users)))
       .service(web::resource("/{id}").route(web::get().to(get_user)))
}

async fn list_users() -> impl Responder {
    HttpResponse::Ok().body("List users is not implemented")
}

// Update your handler function to include the path parameters
async fn get_user(user_info: web::Path<UserInfo>) -> impl Responder {
    let id = &user_info.id; // Now you can use `id` within your function
    HttpResponse::Ok().body(format!("User ID: {}", id))
}

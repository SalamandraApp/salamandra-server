use actix_web::{web, HttpResponse, Responder};
use tokio::task;

use diesel::prelude::*;

use crate::models::exercises::Exercise;
use crate::schema::exercises::dsl::*;


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_exercises))
       .route("/search/{searchTerm}", web::get().to(search_exercise));
}

async fn list_exercises() -> impl Responder {
    "List exercises is not yet implemented"
}

async fn search_exercise(term: web::Path<String>) -> impl Responder {
    let result = task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        exercises.filter(name.like(format!("{}%", term)))
                 .load::<Exercise>(conn)
    }).await;

    match result {
        Ok(Ok(found_exercises)) => HttpResponse::Ok().json(found_exercises),
        Err(_) | Ok(Err(_)) => HttpResponse::InternalServerError().finish(),
    }
}

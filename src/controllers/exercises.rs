use crate::models::exercises::Exercise;
use crate::schema::exercises::dsl::*;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use serde_json::json;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_exercises))
       .route("/search/{searchTerm}", web::get().to(search_exercise));
}

async fn list_exercises() -> impl Responder {
    "List exercises is not yet implemented"
}

async fn search_exercise(term: web::Path<String>) -> impl Responder {
    match web::block( move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        exercises.filter(exercise_name.like(format!("%{}%", term)))
            .load::<Exercise>(conn)

    }).await {
        Ok(Err(DieselError::NotFound)) => {
            HttpResponse::Ok().body("No hay ninguno con ese nombre, ha saltado DieselError::NotFound")
        },
        Ok(Ok(result)) => {
            HttpResponse::Ok().json(result)
        },
        Err(_) | Ok(Err(_)) => {
            HttpResponse::InternalServerError().finish()
        }
    }

}

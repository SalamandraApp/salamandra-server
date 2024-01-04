use actix_web::{web, HttpResponse, Responder};
use tokio::task;
use serde::{Deserialize, Serialize};

use diesel::prelude::*;
use diesel::insert_into;

use crate::models::exercises::{Exercise, NewExercise};
use crate::schema::exercises::dsl as exercises_dsl;

#[derive(Deserialize)]
pub struct AddExercise {
    name: String,
}
#[derive(Serialize)]
struct SearchResponse {
    items: Vec<Exercise>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_exercises))
       .route("", web::post().to(add_exercise))
       .route("/search/{searchTerm}", web::get().to(search_exercise));
}

async fn list_exercises() -> impl Responder {
    "List exercises is not yet implemented"
}


async fn add_exercise(new_exercise: web::Json<AddExercise>) -> HttpResponse {
    let result = tokio::task::spawn_blocking({
        let name_ref = new_exercise.name.clone();
        move || {
            use crate::db::establish_connection;
            let conn = &mut establish_connection();

            let latest_exercise = NewExercise {
                name: name_ref,
            };

            let insert_result = insert_into(exercises_dsl::exercises)
                .values(&latest_exercise)
                .returning((exercises_dsl::id, exercises_dsl::name))
                .get_result::<(i32, String)>(conn);

            match insert_result {
                Ok(num) => Ok(num),
                Err(_) => Err(())
            }
        }
    }).await;

    match result {
        Ok(Ok(n)) => HttpResponse::Ok().json(n),
        Ok(Err(())) | Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


async fn search_exercise(term: web::Path<String>) -> impl Responder {
    let result = task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        exercises_dsl::exercises.filter(exercises_dsl::name.like(format!("{}%", term)))
                 .load::<Exercise>(conn)
    }).await;

    match result {
        Ok(Ok(found_exercises)) => {
            let response = SearchResponse { items: found_exercises };
            HttpResponse::Ok().json(response)
        }
        Err(_) | Ok(Err(_)) => HttpResponse::InternalServerError().finish(),
    }
}

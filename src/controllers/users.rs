use actix_web::{web, Responder, HttpResponse};
use serde_json::json;
use serde::Deserialize;
use tokio::task;
use std::collections::HashSet;

use diesel::prelude::*;
use diesel::{insert_into, sql_query};

use crate::models::workout_linkers::NewWorkoutLinker;
use crate::schema::workout_linkers::dsl::*;
use crate::models::workouts::NewWorkout;
use crate::schema::workouts::dsl as workouts_dsl;
use crate::schema::exercises::dsl as exercises_dsl;



pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_users))
       .route("/{user_id}", web::get().to(get_user))
       .route("/{user_id}/workouts", web::get().to(get_workout))
       .route("/{user_id}/workouts", web::post().to(add_workout));
}

#[derive(Deserialize)]
pub struct ExercisePayload {
    exercise_ids: Vec<u64>,
}

async fn list_users() -> impl Responder {
    "List users is not yet implemented"
}

async fn get_user(_username: web::Path<String>) -> impl Responder {
    "Get user is not yet implemented"
}

async fn get_workout(_username: web::Path<String>) -> impl Responder {
    "Get workouts is not yet implemented"
}

async fn add_workout(
    user_str: web::Path<String>, 
    payload: web::Json<ExercisePayload>) -> HttpResponse {
   
    // Should check JWL token
    // After that should also check that user_id exists
    // For the next phase
    let user: u64 = match user_str.parse::<u64>() {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid exercise_id format"}));
        }
    };
    
    // Remove duplicates
    let unique_ids: HashSet<u64> = payload.exercise_ids.iter().cloned().collect();
    let exercise_id_set: Vec<u64> = unique_ids.into_iter().collect();

    let result = task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        // Count how many of the given ids are in the DB
        let count = exercises_dsl::exercises.filter(exercises_dsl::id.eq_any(exercise_id_set))
            .count()
            .get_result(conn);
        // If different from the ids given, bad request
        if count as usize != payload.exercise_ids.len() {
            return HttpResponse::BadRequest().json(json!({"error": "Some ids don't correspond to exercises"}));
        }
        
        let new_workout = NewWorkout {
            user_id: user,
            // created: date...
        };
        // Insert new workout and get the id it was given
        insert_into(workouts_dsl::workouts)
            .values(&new_workout)          
            .execute(conn);

        let new_workout_id: u64 = match sql_query("SELECT LAST_INSERT_ID()")
            .get_result(conn) {
            Ok(new_id) => new_id,
            Err(_) => {
                return HttpResponse::InternalServerError().finish();
            }
        };

        // Insert linkers with different exe ids, same wk id
        let new_linkers: Vec<NewWorkoutLinker> = exercise_id_set.into_iter().map(|id_value| {
            NewWorkoutLinker {
                exercise_id: id_value,
                workout_id: new_workout_id,
            }
        }).collect();
        insert_into(workout_linkers)
            .values(&new_linkers)
            .execute(conn);

        new_workout_id
    }).await;

    HttpResponse::Ok().body("pum")

}





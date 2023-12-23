use actix_web::{web, Responder, HttpResponse};
use serde_json::json;
use serde::Deserialize;
use tokio::task;
use diesel::prelude::*;
use diesel::insert_into;

use crate::models::exercise_workouts::ExerciseWorkout;
use crate::schema::exercise_workout::dsl::*;

use crate::models::workouts::Workout;
use crate::schema::workouts::dsl::*;
use schema::workouts::dsl as workouts_dsl;

use crate::schema::exercises::dsl::*;
use schema::exercises::dsl as exercises_dsl;


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
    payload: web::Json<ExercisePayload>) -> impl Responder {
   
    let user: u64 = match user_str.parse::<u64>() {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid user_id format. Expected a numeric value"}));
        }
    };

    let id_set: std::collections::HashSet<u64> = payload.exercise_ids.into_iter()
        .filter(|&x| x >= 0)
        .collect();
    let mut ids: Vec<u64> = id_set.into_iter().collect();

    let result = task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        let count = exercises.filter(exercises_dsl::id.eq_any(ids))
            .count()
            .get_result(conn);

        if count as usize != payload.exercise_ids.len() {
            return HttpResponse::BadRequest().json(json!({"error": "List contained a wrong exercise id"}));
        }

        let new_workout_id = insert_into(workouts)
            .values(user_id.eq(user))
            .returning(workouts_dsl::id)            
            .get_result::<u64>(conn)
            .map_err(|e| e.into());
        let values: Vec<_> = ids.iter().map(|&id_value| (exercise_id.eq(id_value), workout_id.eq(new_workout_id))).collect();
        insert_into(exercise_workout)
            .values(&values)
            .execute(conn);

        new_workout_id
    }).await;

    match result {

    }
}





use actix_web::{web, Responder, HttpResponse};
use serde_json::json;
use serde::Deserialize;
use tokio::task;
use std::collections::HashSet;

use diesel::prelude::*;
use diesel::insert_into;

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
    exercise_ids: Vec<i32>,
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

enum WorkoutResult {
    Success(i32),
    BadRequest,
    InternalError,
}

async fn add_workout(
    user_str: web::Path<String>, 
    payload: web::Json<ExercisePayload>) -> HttpResponse {
   
    // Should check JWL token
    // After that should also check that user_id exists
    // For the next phase
    let user: i32 = match user_str.parse::<i32>() {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Non existing exercise ids"}));
        }
    };
    
    // Remove duplicates
    let unique_ids: HashSet<i32> = payload.exercise_ids.iter().cloned().collect();
    let exercise_id_set: Vec<i32> = unique_ids.into_iter().collect();

    let result = task::spawn_blocking(move || {
       
        use crate::db::establish_connection;
        let conn = &mut establish_connection();

        // Count how many of the given ids are in the DB
        let result = exercises_dsl::exercises.filter(exercises_dsl::id.eq_any(exercise_id_set.clone()))
            .count()
            .get_result::<i64>(conn);

        match result {
            
            Ok(count) => {
                if count as usize != payload.exercise_ids.len() {
                    return WorkoutResult::BadRequest;
                }
            },
            Err(_) => {
                return WorkoutResult::BadRequest;
            }
        };

       let new_workout = NewWorkout {
            user_id: user,
            // created: date...
        };
        // Insert new workout and get the id it was given
        let insert_result = insert_into(workouts_dsl::workouts)
            .values(&new_workout)
            .returning(workouts_dsl::id)
            .get_result(conn);
        let new_workout_id = match insert_result {
            Ok(num) => num,
            Err(_) => {
                return WorkoutResult::InternalError;
            }
        };

        // Insert linkers with different exe ids, same wk id
        let new_linkers: Vec<NewWorkoutLinker> = exercise_id_set.into_iter().map(|id_value| {
            NewWorkoutLinker {
                exercise_id: id_value,
                workout_id: new_workout_id,
            }
        }).collect();
        match insert_into(workout_linkers)
            .values(&new_linkers)
            .execute(conn) {
            
            Ok(_) => {},
            Err(_) => {
                return WorkoutResult::InternalError;
            }
        };

        WorkoutResult::Success(new_workout_id)    
    }).await;

    match result {
    Ok(WorkoutResult::Success(new_id)) => HttpResponse::Ok().json(json!({"workout_id": new_id})),
    Ok(WorkoutResult::BadRequest) => HttpResponse::BadRequest().json(json!({"error": "Non existing exercise ids"})),
    Ok(WorkoutResult::InternalError) | Err(_) => HttpResponse::InternalServerError().finish(),
}}





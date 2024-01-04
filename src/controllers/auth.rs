use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use bcrypt::{hash, verify, DEFAULT_COST};
use tokio::task;

use diesel::prelude::*;
use diesel::insert_into;
use diesel::result::Error as DieselError;

use crate::models::users::{User, NewUser};
use crate::schema::users::dsl::*;


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/register", web::post().to(register));
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

// This will have email/phone later
#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}


async fn register(register_info: web::Json<RegisterRequest>) -> HttpResponse {
    let query_result = task::spawn_blocking({
        let username_ref = register_info.username.clone();
        let password_ref = register_info.password.clone();
        move || {
            use crate::db::establish_connection;
            let conn = &mut establish_connection();
            let user_exists = users.filter(username.eq(&username_ref))
                                   .first::<User>(conn)
                                   .is_ok();
            if user_exists {
                Err(DieselError::NotFound) 
            } else {
                let hashed_password = hash(password_ref, DEFAULT_COST).unwrap();

                let new_user = NewUser {
                    username: username_ref,
                    password: hashed_password,
                    // joined: date...
                };

                insert_into(users)
                    .values(&new_user)
                    .execute(conn)
            }
        }
    }).await;

    match query_result {
        Ok(Ok(_)) => HttpResponse::Ok().json(json!({"error": "None", "token": "Your JWT token"})),
        Ok(Err(DieselError::NotFound)) => HttpResponse::BadRequest().body("User already exits"),
        Ok(Err(_)) | Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


async fn login(login_info: web::Json<LoginRequest>) -> HttpResponse {

    let query_result = web::block({
        let username_ref = login_info.username.clone();
        move || {
            use crate::db::establish_connection;
            let conn = &mut establish_connection();

            users.filter(username.eq(username_ref))
                .first::<User>(conn)
        }
    }).await;

    match query_result {
        Ok(Err(DieselError::NotFound)) => {
            HttpResponse::BadRequest().body("User not found")
        },
        Ok(Ok(user)) => {
            match verify(&login_info.password, &user.password) {
                Ok(matches) => {
                    if matches {
                        HttpResponse::Ok().json(json!({"error": "None", "token": "Your JWT token"}))
                    } else {
                        HttpResponse::Unauthorized().body("Incorrect Password")
                    }
                }
                Err(_) => {
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(_) | Ok(Err(_)) => {
            HttpResponse::InternalServerError().finish()
        },
    }
}





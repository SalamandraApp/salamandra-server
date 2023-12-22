use crate::models::users::User;
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use diesel::insert_into;
use diesel::result::Error as DieselError;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use bcrypt::{hash, verify, DEFAULT_COST};

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

    let query_result = web::block({
        let username_ref = register_info.username.clone();
        move || {
            use crate::db::establish_connection;
            let conn = &mut establish_connection();
                users.filter(username.eq(&username_ref))
                .first::<User>(conn)
        }
    }).await;

    match query_result {
        Ok(Err(DieselError::NotFound)) => {            
            let hashed_password = match hash(&register_info.password, DEFAULT_COST) {
                Ok(hashed) => hashed,
                Err(_) => return HttpResponse::InternalServerError().finish(),
            };
            match web::block({
                let username_ref = register_info.username.clone();
                move || {
                    use crate::db::establish_connection;
                    let conn = &mut establish_connection();
                    insert_into(users)
                        .values((username.eq(username_ref), password.eq(hashed_password)))
                        .execute(conn)
                }
            }).await {
                Ok(_) => {},
                Err(_) => return HttpResponse::InternalServerError().finish(),
            }

            HttpResponse::Ok().json(json!({"username": register_info.username}))
        },
        Ok(Ok(_)) => {
            HttpResponse::BadRequest().json(json!({"error": "User already exists"}))
        }
        Err(_) | Ok(Err(_)) => {
            // Error coming out of block
            HttpResponse::InternalServerError().finish()
        },

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
            HttpResponse::BadRequest().json(json!({"error": "User not found"}))
        },
        Ok(Ok(user)) => {
            match verify(&login_info.password, &user.password) {
                Ok(matches) => {
                    if matches {
                        HttpResponse::Ok().json(json!({"token": "Your JWT token"}))
                    } else {
                        HttpResponse::Unauthorized().json(json!({"error": "Incorrect password"}))
                    }
                }
                Err(_) => {
                    // Error during verification
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(_) | Ok(Err(_)) => {
            // Error coming out of block
            HttpResponse::InternalServerError().finish()
        },
    }
}




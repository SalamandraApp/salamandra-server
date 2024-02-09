use actix_web::{web, Responder, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::{execute_db_operation, insert_new_user, select_user};
use crate::models::user::User;
use crate::utils::auth::{handle_protected, ProtectedCallError};
use crate::utils::log::log_db_error;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/{user_id}", web::get().to(get_user));
}

/// Gets user info, inserts it if its new
/// # Arguments
/// * Http request
/// * User id
///
/// # Returns
/// * Unsuccessful http response
/// * User json for new or existing user
pub async fn get_user(req: HttpRequest, url_user_id: web::Path<String>) -> impl Responder {
    
    let claims = match handle_protected(req) {
        Ok(claims) => claims,
        Err(error) => match error {
            ProtectedCallError::WrongHeader => return HttpResponse::BadRequest().json(json!({"error": "Invalid header format"})),
            ProtectedCallError::JwtError(_) => {
                return HttpResponse::Unauthorized().finish()
            }
        }
    };
    let url_uuid = match uuid::Uuid::parse_str(&url_user_id) {
        Ok(url_uuid) => {
            if url_uuid != claims.sub {
                return HttpResponse::Unauthorized().json(json!({"error": "User id mismatch"}))
            }
            url_uuid
        },
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid user id format"}))
        }
    };

    let select_result = match execute_db_operation(Box::new(move |conn| select_user(conn, url_uuid))).await {
        Ok(vec) => vec,
        Err(error) => {
            log_db_error(error);
            println!("Error selecting");
            return HttpResponse::InternalServerError().finish()
        }
    };

    // New user
    let len = select_result.len();
    print!("This many users {}", len);
    if len == 0 {
        let user_name = claims.preferred_username.clone(); 
        let new_user = User {
            id: claims.sub,
            username: user_name.clone(),
            display_name: user_name,        
            date_joined: chrono::Utc::now(),
            training_state: 0,
            fitness_level: 0,
            pfp_url: None,
            date_of_birth: None,
            height: None,
        };

        // Insert into db
        let user_db = new_user.clone();
        match execute_db_operation(Box::new(move |conn| insert_new_user(conn, user_db))).await {
            Ok(_) => HttpResponse::Created().json(new_user),
            Err(error) => {
                log_db_error(error);
                println!("Error inserting");
                HttpResponse::InternalServerError().finish()
            }
        }

    }
    else if len == 1 {
        let user = select_result.first().unwrap(); 
        HttpResponse::Ok().json(user)
    }
    else {
        // TODO: logging
        println!("Error selecting more than 2");
        HttpResponse::InternalServerError().finish()
    }
}

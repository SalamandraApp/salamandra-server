use actix_web::{web, Responder, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::{execute_db_operation, insert_new_user, select_user};
use crate::models::user::*;
use crate::utils::auth::{handle_protected_call, ProtectedCallError};
use crate::utils::log::log_db_error;
use crate::Config;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/{user_id}", web::get().to(get_user))
        .route("/search", web::get().to(search_users));
}

/// Gets user info, inserts it if its new
/// # Arguments
/// * Http request
/// * User id
///
/// # Returns
/// * Unsuccessful http response
/// * User json for new or existing user
pub async fn get_user(req: HttpRequest, url_user_id: web::Path<String>, config: web::Data<Config>) -> impl Responder {
   
    let claims = match handle_protected_call(req, &config.jwks_url, &config.issuer).await {
        Ok(claims) => claims,
        Err(error) => match error {
            // TODO: logging
            ProtectedCallError::WrongHeader => return HttpResponse::BadRequest().json(json!({"error": "Invalid header format"})),
            ProtectedCallError::JwtError(mes) => {
                println!("JWT Error: {}", mes);
                return HttpResponse::Unauthorized().finish()
            },
            ProtectedCallError::ErrorGettingKey(mes) => {
                println!("Error Getting Key: {}", mes);
                return HttpResponse::Unauthorized().finish()
            }
        }
    };
    let url_uuid = match uuid::Uuid::parse_str(&url_user_id) {
        Ok(url_uuid) => url_uuid,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Invalid user id format"})),
    };

    let db_url = config.db_url.clone();
    let select_result = match execute_db_operation(Box::new(move |conn| select_user(conn, url_uuid)), db_url).await {
        Ok(vec) => vec,
        Err(error) => {
            log_db_error(error);
            return HttpResponse::InternalServerError().finish()
        }
    };

    // New user
    // TODO: 
    // - change visibility of user struct if not getting own profile
    let len = select_result.len();
    if len == 0 {
        // Looking for different user than one calling
        if claims.sub != url_uuid {
            return HttpResponse::NotFound().json(json!({"error" : "User not found"}))
        }
        let user_name = claims.nickname.clone(); 
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
        let db_url = config.db_url.clone();
        match execute_db_operation(Box::new(move |conn| insert_new_user(conn, user_db)), db_url).await {
            Ok(_) => HttpResponse::Created().json(new_user),
            Err(error) => {
                log_db_error(error);
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


/// Retrieve list of users based on a term
/// # Arguments
/// * /search?username=value
///
/// # Returns
/// * List of matching (username, uuid) pairs
pub async fn search_users(query: web::Query<UserSearchParams>) -> impl Responder {
    let _username = &query.username;
    HttpResponse::NotImplemented().finish()
}

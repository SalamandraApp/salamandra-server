use actix_web::{web, Responder, HttpRequest, HttpResponse};
use serde_json::json;
use tokio::task;
use tokio::time::{self, Duration};
use tokio::sync::Mutex;
use std::sync::Arc;

use diesel::prelude::*;
use diesel::{insert_into, delete};

use crate::schema::users::dsl::*;
use crate::models::user::User;
use crate::utils::auth::{process_jwt, AccessTokenClaims};
use crate::utils::keycloak::KeycloakClient;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/{user_id}", web::get().to(get_user))
       .route("", web::post().to(add_user));
}

pub async fn get_user(_username: web::Path<String>) -> impl Responder {
    HttpResponse::NotImplemented().json(json!({"error": "Funcionality not yet implemented"}))
}


/// Adds the user to the database 
/// and schedules email verification check
///
/// # Arguments
/// * JWT token in request
///
/// # Errors
/// * Invalid/Unauthorized JWT
/// * Invalid UUID
/// * Expired token
/// * Internal Server Errors
pub async fn add_user(req: HttpRequest, 
                  keycloak_client: web::Data<Arc<Mutex<KeycloakClient>>>) -> HttpResponse {

    // Extract header
    let auth_header = match req.headers().get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
            Some(header) => header,
            None => return HttpResponse::BadRequest().finish(),
        };

    // Validate claims
    let claims: AccessTokenClaims = match process_jwt(auth_header) {
        Ok(claims) => claims,
        // Generalizing (for now), assuming all errors are unauthorized client side
        Err(jwt_error) => return HttpResponse::Unauthorized().json(json!({"error" : jwt_error.to_string()}))
    };

    // Transform uuid
    let uuid = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(user_id) => user_id,
        Err(_) => return HttpResponse::BadRequest().json(json!({"error": "Wrongly formatted uuid"}))
    };
    
    // Check expiration
    let current_time = chrono::Utc::now();
    if current_time.timestamp() >= claims.exp {
        return HttpResponse::Unauthorized().json(json!({"error": "Token is expired"}));
    }

    // Create user
    let user_name= claims.preferred_username.clone(); 
    let new_user = User {
        id: uuid,
        username: user_name.clone(),
        display_name: user_name,        
        date_joined: current_time,
        training_state: 0,
        fitness_level: 0,
        pfp_url: None,
        date_of_birth: None,
        height: None,
    };
     
    // Insert into db
    let operation_result = task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn_result = establish_connection(); 
        let mut conn = match conn_result {
            Ok(connection) => connection,
            Err(conn_err) => return Err(conn_err)
        };
         
        Ok(insert_into(users)
            .values(&new_user)
            .execute(&mut conn))
            
    }).await;

    match operation_result {
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Unexpected error"})),
        // Error with connection or insert
        Ok(Err(_)) => HttpResponse::InternalServerError().json(json!({"error": "Service unavailable"})),
        Ok(_) => {
            let user_name = claims.preferred_username.clone();
            tokio::spawn(async move {
                time::sleep(Duration::from_secs(12 * 3600)).await;
                check_email_verified(user_name, keycloak_client).await;
            });

            HttpResponse::Created().finish()
        }
    }
}

/// Gets user info from KC and checks email verified
/// If its not, deletes the user from KCs and our db
/// Otherwise nothing
///
/// # Argument
/// * username
async fn check_email_verified(user_name: String, keycloak_client: web::Data<Arc<Mutex<KeycloakClient>>>) {

    // Call KC
    let mut client = keycloak_client.lock().await;
    let user_id = match client.get_user_info(user_name).await {
        Ok(info) => {
            if info.email_verified {
                return
            }
            info.id
        },
        Err(_) => return,
    };

    // Delete from KC
    match client.delete_user(user_id.clone()).await {
        Ok(_) => (),
        // TODO
        // logging
        Err(_) => (),
    }

    // Delete from out DB
    let user_uuid = match uuid::Uuid::parse_str(&user_id) {
        Ok(user_id) => user_id,
        // TODO
        // logging
        Err(_) => return,
    };

    match task::spawn_blocking(move || {
        use crate::db::establish_connection;
        let conn_result = establish_connection(); 
        let mut conn = match conn_result {
            Ok(connection) => connection,
            Err(conn_err) => return Err(conn_err)
        };

        Ok(delete(users.filter(id.eq(user_uuid))).execute(&mut conn))
    }).await {
        Ok(_) => (),
        Err(_) => (),
    };
}

use actix_web::{web, Responder, HttpRequest, HttpResponse};
use serde_json::json;
use tokio::time::{self, Duration};
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::db::{execute_db_operation, insert_new_user, delete_user};

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

    // Create user
    let user_name= claims.preferred_username.clone(); 
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
    let operation_result = execute_db_operation(Box::new(|| insert_new_user(new_user))).await;
    
    match operation_result {
        Ok(_) => {
            let user_name = claims.preferred_username.clone();
            tokio::spawn(async move {
                time::sleep(Duration::from_secs(12 * 3600)).await;
                check_email_verified(user_name, keycloak_client).await;
            });

            HttpResponse::Created().finish()
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
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

    // Delete from out DB
    let user_id_clone = user_id.clone();
    let operation_result = execute_db_operation(Box::new(move || delete_user(user_id_clone))).await;
    // TODO
    // logging
    match operation_result {
        Ok(_) => (),
        Err(_) => (),
    };

    // Delete from KC
    match client.delete_user(user_id.to_string()).await {
        Ok(_) => (),
        // TODO
        // logging
        Err(_) => (),
    }
}

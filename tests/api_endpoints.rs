//! API Endpoints Testing Module
//!
//! The primary focus is to ensure that each endpoint consistently returns 
//! the expected status codes and response formats. 
//! These tests are designed to validate:
//! - Predictable responses
//! - Status codes
//! - Response formats
//!
//! NOTE: These tests do not cover the internal logic or accuracy of the endpoints. 
//! Only the interface layer, the communication of endpoints with clients.

use actix_web::{web, App, test};
use reqwest::StatusCode;
use reqwest::header::AUTHORIZATION;
use salamandra_server::handlers::users::get_user;

mod common;


#[actix_web::test]
async fn test_get_user_wrong_header() {
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // No authorization header
    let req_1 = test::TestRequest::get()
        .uri("/users/some_id")
        .to_request();
    let resp_1 = test::call_service(&app, req_1).await;

    // Wrong format in authorization header
    let req_2 = test::TestRequest::get()
        .uri("/users/some_id")
        .insert_header((AUTHORIZATION, "No Bearer tokenTOKEN"))
        .to_request();
    let resp_2 = test::call_service(&app, req_2).await;

    // Incorrect token
    let req_3 = test::TestRequest::get()
        .uri("/users/some_id")
        .insert_header((AUTHORIZATION, "Bearer notatokenatall"))
        .to_request();
    let resp_3 = test::call_service(&app, req_3).await;

    assert_eq!(resp_1.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp_2.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp_3.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_user_invalid_jwt() {

}




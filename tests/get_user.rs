//! Testing handlers/users/ get_user
//!
//! I don't know if it will stay here, but its good for now
//!
//! Test Naming
//! test_<function>_<case>

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
async fn test_get_user_wrong_url_user_id() {
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    let private_key = common::set_up_test_key();
    let token = common::get_test_token(private_key, None);

    // Not uuid format in URL
    let req_1 = test::TestRequest::get()
        .uri("/users/{WRONG-UUID}")
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp_1 = test::call_service(&app, req_1).await;

    // Not uuid format in url
    let req_2 = test::TestRequest::get()
        .uri(format!("/users/{}", uuid::Uuid::new_v4().to_string()).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp_2 = test::call_service(&app, req_2).await;

    assert_eq!(resp_1.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp_2.status(), StatusCode::UNAUTHORIZED);
    common::clean_up_test_key();
}


#[actix_web::test]
async fn test_get_user_new_user() {
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    let private_key = common::set_up_test_key();
    let token = common::get_test_token(private_key, None);
    

    let req = test::TestRequest::get()
        .uri(format!("/users/{}", common::NEW_TEST_UUID).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), reqwest::StatusCode::CREATED); 

    common::remove_user(None).await;
    common::clean_up_test_key(); 
}


#[actix_web::test]
async fn test_get_user_existing_user() {
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    let private_key = common::set_up_test_key();
    let existing_user = common::insert_user().await;
    let user_id = existing_user.id.clone();
    let token = common::get_test_token(private_key, Some(existing_user));
    

    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_id).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), reqwest::StatusCode::OK); 

    common::remove_user(Some(user_id)).await;
    common::clean_up_test_key(); 
}

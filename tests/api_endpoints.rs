use actix_web::{web, App, test};
use reqwest::StatusCode;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use tokio::sync::Mutex;
use std::sync::Arc;
use salamandra_server::handlers::users::{get_user, add_user};
use salamandra_server::utils::keycloak::KeycloakClient;

#[actix_web::test]
async fn test_get_user() {
    let app = test::init_service(
            App::new().route("/users/{user_id}", web::get().to(get_user))
        ).await;
    let req = test::TestRequest::get()
            .uri("/users/example_id")
            .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);

    let result: ErrorMessage = test::read_body_json(resp).await;
    assert_eq!(result.error, "Funcionality not yet implemented");
}

#[actix_web::test]
async fn test_add_user_bad_request() {
    let keycloak_client = Arc::new(Mutex::new(KeycloakClient {
        token: None,
        token_expires: 0,
    })); 
    let app = test::init_service(
            App::new()
                .app_data(web::Data::new(keycloak_client.clone()))
                .route("/users", web::post().to(add_user))
        ).await;


    let req_1 = test::TestRequest::post()
        .uri("/users")
        .to_request();
    let resp_1 = test::call_service(&app, req_1).await;

    let req_2 = test::TestRequest::post()
        .uri("/users")
        .insert_header((AUTHORIZATION, "No Bearer tokenTOKEN"))
        .to_request();
    let resp_2 = test::call_service(&app, req_2).await;
   
    let req_3 = test::TestRequest::post()
        .uri("/users")
        .insert_header((AUTHORIZATION, "Bearer notatokenatall"))
        .to_request();
    let resp_3 = test::call_service(&app, req_3).await;


    assert_eq!(resp_1.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp_2.status(), StatusCode::BAD_REQUEST);
    assert_eq!(resp_3.status(), StatusCode::UNAUTHORIZED);
}

#[derive(Deserialize)]
struct ErrorMessage {
    error: String
}



//! Testing handlers/users/ get_user
//!
//! I don't know if it will stay here, but its good for now
//!
//! Test Naming
//! test_<function>_<case>

use actix_web::{web, App, test};
use reqwest::StatusCode;
use jsonwebtoken as jwt;
use mockito::Server;

use salamandra_server::handlers::users::get_user;
use salamandra_server::utils::{test as common, auth::AccessTokenClaims};

#[actix_web::test]
async fn test_get_user_same_user_new_sucess() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}
#[actix_web::test]
async fn test_get_user_same_user_existing_sucess() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::OK);
}
#[actix_web::test]
async fn test_get_user_different_user_existing_sucess() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::OK);
}



#[actix_web::test]
async fn test_get_user_invalid_header() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("WRONG {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.remove_async().await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
#[actix_web::test]
async fn test_get_user_invalid_validation() {
    // Create token
    let mut server = Server::new_async().await;
    let (_private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let (other_private_key, _, _, _, _) = common::generate_key();
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&other_private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
#[actix_web::test]
async fn test_get_user_error_getting_jwks() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, _kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: None,
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.remove_async().await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}



#[actix_web::test]
async fn test_get_user_invalid_url_uuid() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri("/users/WRONG_UUID")
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_get_user_different_user_doesnt_exist() {
    // Create token
    let mut server = Server::new_async().await;
    let (private_key, kid, mock) = common::set_up_jwks_endpoint(&mut server, 5).await;
    let now = chrono::Utc::now().timestamp() as u64; 
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test".to_owned(),
        aud: "test".to_owned(),
        exp: now + 100,
        iat: now - 100,
        token_use: "id".to_owned(),
    };
    let header = jwt::Header {
        alg: jwt::Algorithm::RS256,
        kid: Some(kid),
        ..Default::default()
    };
    let token = jwt::encode(
        &header,
        &claims,
        &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

    // Create test server
    let app = test::init_service(
        App::new()
        .route("/users/{id}", web::get().to(get_user))
        ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", uuid::Uuid::new_v4()).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    mock.assert_async().await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/*
#[actix_web::test]
async fn test_get_user_error_selecting() {
}

#[actix_web::test]
async fn test_get_user_error_inserting() {
}
*/

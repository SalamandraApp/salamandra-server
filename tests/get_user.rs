//! Testing handlers/users/ get_user
//!get
//! I don't know if it will stay here, but its good for now
//!
//! Test Naming
//! test_<function>_<case>
use actix_web::{web, App, test};
use reqwest::StatusCode;
use jsonwebtoken as jwt;
use mockito::Server;
use testcontainers::clients::Cli;

use salamandra_server::handlers::users::get_user;
use salamandra_server::db::establish_connection;
use salamandra_server::Config;
use salamandra_server::utils::{test as common, auth::AccessTokenClaims};


#[actix_web::test]
async fn test_get_user_same_user_new_sucess() {
    
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url,"test_issuer".to_string(),jwks_url);

    // Create token
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
        .route("/users/{id}", web::get().to(get_user))
    ).await;

    // Send request
    let req = test::TestRequest::get()
        .uri(format!("/users/{}", user_uuid).as_str())
        .insert_header((reqwest::header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    mock.assert_async().await;
}
#[actix_web::test]
async fn test_get_user_same_user_existing_sucess() {
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);

    // Create token
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1, db_url).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);

    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1, db_url).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (_private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);

    let (other_private_key, _, _, _, _) = common::generate_key();
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = common::insert_users(1, db_url).await.into_iter().next().unwrap();
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, _kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);
    let now = chrono::Utc::now().timestamp() as u64; 
    let user_uuid = uuid::Uuid::new_v4();
    let claims = AccessTokenClaims {
        sub: user_uuid,
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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
    // Set up docker
    let docker = Cli::default();
    let image = common::container_setup();
    let instance = docker.run(image.image);
    let db_url = format!(
        "postgres://{}:password@127.0.0.1:{}/{}",
        image.user,
        instance.get_host_port_ipv4(5432),
        image.db
    );
    let mut conn = establish_connection(db_url.clone()).unwrap();
    common::run_migrations(&mut conn)
        .expect("Failed to run migrations");
    // Set up endpoint
    let mut server = Server::new_async().await;
    let (private_key, kid, mock, jwks_url) = common::set_up_jwks_endpoint(&mut server, 5).await;
    
    // App config
    let config = Config::new(db_url.clone(),"test_issuer".to_string(),jwks_url);
    let now = chrono::Utc::now().timestamp() as u64; 
    let claims = AccessTokenClaims {
        sub: uuid::Uuid::new_v4(),
        nickname: "test".to_owned(),
        email_verified: true,
        iss: "test_issuer".to_owned(),
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
        .app_data(web::Data::new(config))
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

use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::keycloak::KeycloakClient;

pub mod db;
pub mod utils;
pub mod models;
pub mod schema;
pub mod handlers;

pub async fn run() -> std::io::Result<()> {
    /*
    FOR DEPLOYMENT
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("~/backend-certs/privkey.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("~/backend-certs/fullchain.pem").unwrap(); 
    */

    
    let keycloak_client = Arc::new(Mutex::new(KeycloakClient {
        token: None,
        token_expires: 0,
    })); 
    
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("keys/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("keys/cert.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(keycloak_client.clone()))
            .service(web::scope("/users").configure(handlers::users::config))
    })
    .bind_openssl("localhost:8080", builder)?
    .run()
    .await
}

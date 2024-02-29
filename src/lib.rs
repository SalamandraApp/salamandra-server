use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

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
 
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("keys/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("keys/cert.pem").unwrap();

    let config = Config::load();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .service(web::scope("/users").configure(handlers::users::config))
    })
    .bind_openssl("localhost:8080", builder)?
    .run()
    .await
}

#[derive(Clone)]
pub struct Config {
    pub db_url: String,
    pub issuer: String,
    pub jwks_url: String
}

impl Config {
    pub fn new(db_url: String, issuer: String, jwks_url: String) -> Self {
        Config{
            db_url,
            issuer,
            jwks_url 
        }
    }

    pub fn load() -> Self {
        dotenv::dotenv().ok(); 

        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let issuer = std::env::var("ISSUER").expect("ISSUER must be set");
        let jwks_url = std::env::var("JWKS_URL").expect("JWKS_URLmust be set");

        Config{
            db_url,
            issuer,
            jwks_url 
        }
    }
}

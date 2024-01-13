use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod db;
mod utils;
mod models;
mod schema;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    /*
    FOR DEPLOYMENT
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("~/backend-certs/privkey.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("~/backend-certs/fullchain.pem").unwrap();
     
    FOR TESTING 
    `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    */
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("keys/key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("keys/cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(web::scope("/users").configure(handlers::users::config))
    })
    .bind_openssl("localhost:8080", builder)?
    .run()
    .await
}

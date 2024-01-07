use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod db;
mod models;
mod schema;
mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    /*
    FOR DEPLOYMENT
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("~/backend-certs/privkey.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("~/backend-certs/fullchain.pem").unwrap();
    */

    /*
    FOR TESTING 
    `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    */
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| {
        App::new()
            .service(web::scope("/users").configure(controllers::users::config))
            .service(web::scope("/auth").configure(controllers::auth::config))
            .service(web::scope("/exercises").configure(controllers::exercises::config))
    })
    .bind_openssl("localhost:8080", builder)?
    .run()
    .await
}

use actix_web::{web, App, HttpServer};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod db;
mod models;
mod schema;
mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("~/backend-certs/privkey.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("~/backend-certs/fullchain.pem").unwrap();
    HttpServer::new(|| {
        App::new()
            .service(web::scope("/users").configure(controllers::users::config))
            .service(web::scope("/auth").configure(controllers::auth::config))
            .service(web::scope("/exercises").configure(controllers::exercises::config))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

use actix_web::{web, App, HttpServer};
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
mod db;
mod models;
mod schema;
mod controllers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("rust_log", "actix_web=debug");

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

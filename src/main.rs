mod users;

use actix_web::{web, App, HttpServer};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    HttpServer::new(|| {
        App::new()
            .configure(users::config)
            .configure(auth::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


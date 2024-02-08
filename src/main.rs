use salamandra_server::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize log4rs");
    run().await
}

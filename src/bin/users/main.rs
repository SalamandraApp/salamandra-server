mod create_user;
mod get_user;
mod search_users;

use get_user::get_user;
use create_user::create_user;
use search_users::search_users;

use lambda_http::{run, service_fn, Error, Request, Response, Body, tracing};
use lambda_http::http::Method;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let handler = service_fn(|event| router(event));
    run(handler).await
}

async fn router(event: Request) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let response = match (event.method(), path) {
        (&Method::GET, _) if Regex::new(r"^/users/[a-fA-F0-9-]+$").unwrap().is_match(path) => get_user(event, None).await,
        (&Method::POST, "/users") => create_user(event, None).await,
        (&Method::GET, "/users") => search_users(event, None).await,
        _ => {
            println!("Unmatched route: Method: {}, URI: {}", event.method(), event.uri().path());
            Ok(Response::builder()
                .status(404)
                .body("Not Found".into())
                .expect("Failed to render response"))
        }
    };
    response
}


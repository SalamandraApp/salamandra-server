mod get_exercise;
mod search_exercises;

use get_exercise::get_exercise;
use search_exercises::search_exercises_;

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
        (&Method::GET, _) if Regex::new(r"^/exercises/\w+$").unwrap().is_match(path) => get_exercise(event, None).await,
        (&Method::GET, "/exercises") => search_exercises_(event, None).await,
        _ => Ok(Response::builder()
                .status(404)
                .body("Not Found".into())
                .expect("Failed to render response")),
    };

    response
}


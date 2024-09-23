mod get_exercise;
mod search_exercises;

use get_exercise::get_exercise;
use salamandra_server::lib::utils::handlers::{not_found, UUID_PATTERN};
use search_exercises::search_exercises_;
use salamandra_server::lib::db::DBConnector;

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
    let connector = DBConnector::default();
    let response = match (event.method(), path) {
        (&Method::GET, _) if Regex::new(&format!(r"^/exercises/{}$", UUID_PATTERN)).unwrap().is_match(path) => get_exercise(event, &connector).await,
        (&Method::GET, "/exercises") => search_exercises_(event, &connector).await,
        _ => not_found()
    };

    response
}


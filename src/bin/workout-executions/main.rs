mod get_workout_execution;
mod create_workout_execution;

use create_workout_execution::create_workout_execution;
use get_workout_execution::get_workout_execution;

use salamandra_server::lib::db::DBConnector;

use lambda_http::{run, service_fn, Error, Request, Response, Body, tracing};
use lambda_http::http::Method;
use regex::Regex;
use salamandra_server::lib::utils::handlers::{not_found, UUID_PATTERN};

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
        (&Method::POST, _) if Regex::new(&format!(r"^/users/{}/workout-executions$", UUID_PATTERN)).unwrap().is_match(path) => create_workout_execution(event, &connector).await,
        (&Method::GET, _) if Regex::new(&format!(r"^/users/{}/workout-executions/{}$", UUID_PATTERN, UUID_PATTERN)).unwrap().is_match(path) => get_workout_execution(event, &connector).await,
        _ => not_found()
    };
    response
}


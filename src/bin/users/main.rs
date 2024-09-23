mod create_user;
mod get_user;
mod search_users;
mod patch_user;

use get_user::get_user;
use create_user::create_user;
use search_users::search_users;
use patch_user::patch_user;
use salamandra_server::lib::utils::handlers::{not_found, UUID_PATTERN};
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
    let specific_user = Regex::new(&format!(r"^/users/{}$", UUID_PATTERN));
    let response = match (event.method(), path) {
        (&Method::POST, "/users") => create_user(event, &connector).await,
        (&Method::GET, _) if specific_user.clone().unwrap().is_match(path) => get_user(event, &connector).await,
        (&Method::PATCH, _) if specific_user.unwrap().is_match(path) => patch_user(event, &connector).await,
        (&Method::GET, "/users") => search_users(event, &connector).await,
        _ => not_found()
    };
    response
}


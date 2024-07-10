mod create_workout_template;
mod delete_workout_template;
mod get_all_workout_templates;
mod get_workout_template;

use create_workout_template::create_workout_template;
use delete_workout_template::delete_workout_template_;
use get_all_workout_templates::get_all_workout_templates;
use get_workout_template::get_workout_template;

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
        (&Method::POST, _) if Regex::new(r"^/users/[a-fA-F0-9-]+$/workout-templates").unwrap().is_match(path) => create_workout_template(event, None).await,
        (&Method::DELETE, _) if Regex::new(r"^/users/[a-fA-F0-9-]+$/workout-templates/[a-fA-F0-9-]+$").unwrap().is_match(path) => delete_workout_template_(event, None).await,
        (&Method::GET, _) if Regex::new(r"^/users/[a-fA-F0-9-]+$/workout-templates").unwrap().is_match(path) => get_all_workout_templates(event, None).await,
        (&Method::GET, _) if Regex::new(r"^/users/[a-fA-F0-9-]+$/workout-templates/[a-fA-F0-9-]+$").unwrap().is_match(path) => get_workout_template(event, None).await,
        _ => Ok(Response::builder()
                .status(404)
                .body("Not Found".into())
                .expect("Failed to render response")),
    };

    response
}


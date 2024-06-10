use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

use salamandra_server::lib::models::user_models::User;

#[derive(Deserialize)]
struct Request {
}


#[derive(Serialize)]
struct Response {
    statusCode: i32,
    body: String,
}


async fn get_user(_event: LambdaEvent<Request>) -> Result<User, Error> {

    let resp = User::default();
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(get_user)).await
}

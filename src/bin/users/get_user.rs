use lambda_http::{run, service_fn, Error, Request, Response, Body, RequestExt, http::StatusCode, tracing};
use uuid::Uuid;

// use salamandra_server::lib::models::user_models::User;
use salamandra_server::lib::db::users_db::lookup_user;
use salamandra_server::lib::utils::build_resp;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::errors::DBError;


async fn get_user(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {
    
    let path_parameters = event.path_parameters();
    let user_id = path_parameters
        .first("user_id")
        .unwrap_or("No ID provided");
    let id = Uuid::parse_str(user_id).expect("Error");

    match lookup_user(id, test_db).await {
        Ok(user) => Ok(build_resp(StatusCode::OK, user)),
        Err(DBError::ItemNotFound(mes)) => Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(_) => Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let handler = service_fn(|event| get_user(event, None));
    run(handler).await
}


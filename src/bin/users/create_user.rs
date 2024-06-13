use lambda_http::{run, service_fn, Error, Request, Response, Body, tracing};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use salamandra_server::lib::models::user_models::User;
use salamandra_server::lib::db::users_db::insert_user;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::errors::DBError;
use salamandra_server::lib::utils::handlers::build_resp;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub uuid: Uuid,
    pub username: String,
}

async fn create_user(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    if let Body::Text(body) = event.into_body() {
        if let Ok(req) = serde_json::from_str::<CreateUserRequest>(&body) {
            let new_user = User {
                id: req.uuid,
                username: req.username,
                ..Default::default()
            };
            let resp = match insert_user(&new_user, test_db).await {
                Ok(user) => build_resp(StatusCode::CREATED, user),
                Err(DBError::UniqueViolation(mes)) => build_resp(StatusCode::CONFLICT, mes),
                Err(_) => build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""),
            };
            return Ok(resp);
        }
    }

    Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload"))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let handler = service_fn(|event| create_user(event, None));
    run(handler).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;
    use uuid::Uuid;
    use salamandra_server::lib::utils::tests::pg_container;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MissingFields {
        pub field: i32,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct DifferentTypes {
        pub uuid: i32,
        pub username: String,
    }


    #[tokio::test]
    async fn test_create_user_invalid_payload() {
        let (pool, _container) = pg_container().await;

        {   // ------ Different fields
            let payload = MissingFields {field: 1};

            let mut req = Request::default();
            *req.uri_mut() = "/users".parse().unwrap();
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));

            let resp = create_user(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        {   // ------ Different Types 
            let payload = DifferentTypes {
                uuid: 1,
                username: "username".to_string(),
            };

            let mut req = Request::default();
            *req.uri_mut() = "/users".parse().unwrap();
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));

            let resp = create_user(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }        

        {   // ------  No Payload

            let mut req = Request::default();
            *req.uri_mut() = "/users".parse().unwrap();

            let resp = create_user(req, Some(pool)).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }
    
    #[tokio::test]
    async fn test_create_user_existing() {
        let (pool, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let payload = CreateUserRequest {
            uuid: user_id,
            username: "username".to_string(),
        };

        let mut req = Request::default();
        *req.uri_mut() = "/users".parse().unwrap();
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        
        let resp = create_user(req.clone(), Some(pool.clone())).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let resp = create_user(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    
    #[tokio::test]
    async fn test_create_user_new() {        
        let (pool, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let payload = CreateUserRequest {
            uuid: user_id,
            username: "username".to_string(),
        };

        let mut req = Request::default();
        *req.uri_mut() = "/users".parse().unwrap();
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        
        let resp = create_user(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}

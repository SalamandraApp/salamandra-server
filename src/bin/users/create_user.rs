use lambda_http::{Error, Request, Response, Body};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;
use chrono::NaiveDate;

use salamandra_server::lib::models::user_models::User;
use salamandra_server::lib::db::users_db::insert_user;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::errors::DBError;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    uuid: Uuid,
    username: String,
    date_joined: NaiveDate,
}

pub async fn create_user(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    if let Body::Text(body) = event.clone().into_body() {
        if let Ok(req) = serde_json::from_str::<CreateUserRequest>(&body) {
            match extract_sub(event.headers(), Some(req.uuid)) {
                Ok(_) => (),
                Err(resp) => return Ok(resp)
            };
            let new_user = User {
                id: req.uuid,
                username: req.username,
                date_joined: req.date_joined,
                ..Default::default()
            };
            let resp = match insert_user(&new_user, test_db).await {
                Ok(user) => build_resp(StatusCode::CREATED, user),
                Err(DBError::UniqueViolation(mes)) => {
                    warn!("Tried to insert already exisiting user");
                    build_resp(StatusCode::CONFLICT, mes)
                },
                Err(error) => {
                    error!("INTERNAL SERVER ERROR: {}", error);
                    build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")
                }
            };
            return Ok(resp);
        }
    }

    Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload"))
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use lambda_http::http::{header::AUTHORIZATION, HeaderValue};
    use serde_json::to_string;
    use uuid::Uuid;
    use salamandra_server::lib::utils::tests::{pg_container, test_jwt};

    #[derive(Debug, Serialize, Deserialize)]
    struct MissingFields {
        field: i32,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    struct DifferentTypes {
        uuid: i32,
        username: String,
        date_joined: String,
    }


    #[tokio::test]
    async fn test_create_user_invalid_payload() {
        let (pool, _container) = pg_container().await;
        let user_id = Uuid::new_v4();

        {   // ------ Different fields
            let payload = MissingFields {field: 1};

            let mut req = Request::default();
            let jwt = test_jwt(user_id);
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
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
                date_joined: "date".to_string(),
            };

            let mut req = Request::default();
            let jwt = test_jwt(user_id);
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));

            let resp = create_user(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }        

        {   // ------  No Payload

            let mut req = Request::default();
            let jwt = test_jwt(user_id);
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
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
            date_joined: Utc::now().naive_utc().date(),
        };

        let mut req = Request::default();
        let jwt = test_jwt(user_id);
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
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
            date_joined: Utc::now().naive_utc().date(),
        };
        println!("DATE: {}", Utc::now().naive_utc().date());
        let mut req = Request::default();
        let jwt = test_jwt(user_id);
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        
        let resp = create_user(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}

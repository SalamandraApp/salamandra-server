use lambda_http::{Error, Request, Response, Body, RequestExt,};
use lambda_http::http::StatusCode;
use tracing::{error, info, warn};
use uuid::Uuid;

use salamandra_server::lib::db::users_db::update_user;
use salamandra_server::lib::models::user_models::UncompleteUser;
use salamandra_server::lib::errors::DBError;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBConnector;

/// Update user
/// * Update certain values of an existing user
pub async fn patch_user(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    if let Body::Text(body) = event.clone().into_body() {
        if let Ok(req) = serde_json::from_str::<UncompleteUser>(&body) {
            let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();
            // Check user is updating themselves
            match extract_sub(event.headers(), Some(user_id)) {
                Ok(_) => (),
                Err(resp) => return Ok(resp)
            };

            let res = match update_user(&user_id, &req, connector).await {
                Ok(updated_user) => build_resp(StatusCode::OK, updated_user),
                Err(DBError::ItemNotFound(mes)) => {
                    warn!("Tried to update non-existing user");
                    build_resp(StatusCode::NOT_FOUND, mes)
                }
                Err(DBError::QueryError(mes)) => {
                    println!("ERROR: {}", mes);
                    info!("Didn't modify user: {}", mes);
                    build_resp(StatusCode::NOT_MODIFIED, "")
                }
                Err(error) => {
                    error!("INTERNAL SERVER ERROR: {}", error);
                    build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")
                }
            };
            return Ok(res)
        }
    }
    Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    use lambda_http::http::{header::AUTHORIZATION, HeaderValue};
    use serde_json::to_string;
    use salamandra_server::lib::utils::tests::{insert_helper, pg_container, test_jwt, Items};


    // TEST CASES
    // * Invalid payload
    // * Try to update new user
    // * Success
    
    #[tokio::test]
    async fn test_patch_user_invalid_payload() {
        let (connector, _container) = pg_container().await;

        let user_id = Uuid::new_v4();
        let mut req = Request::default();
        
        let jwt = test_jwt(user_id);
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());

        let resp = patch_user(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_patch_user_new_user() {
        let (connector, _container) = pg_container().await;

        let id = Uuid::new_v4();
        let mut req = Request::default();
        
        let user_id = id.to_string();
        let payload = UncompleteUser{..Default::default()};
        let jwt = test_jwt(id);
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));

        let resp = patch_user(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }
    #[tokio::test]
    async fn test_patch_user_success() {
        let (connector, _container) = pg_container().await;

        let mut req = Request::default();
        let id = insert_helper(1, Items::Users, &connector, None).await[0];

        let payload = UncompleteUser{display_name: Some("New".to_string()), ..Default::default()};
        let jwt = test_jwt(id);
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), id.to_string())]));

        let resp = patch_user(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

use lambda_http::{Error, Request, Response, Body, RequestExt,};
use lambda_http::http::StatusCode;
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::users_db::lookup_user;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::errors::DBError;

/// Fetch user
/// * Assumes user id has been previously checked
pub async fn get_user(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();

    // Fetch from database
    match lookup_user(user_id, connector).await {
        Ok(user) => Ok(build_resp(StatusCode::OK, user)),
        Err(DBError::ItemNotFound(mes)) => Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(error) => {
            error!("INTERNAL SERVER ERROR: {}", error);
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    use lambda_http::http::StatusCode;
    use salamandra_server::lib::utils::tests::pg_container;
    use salamandra_server::lib::models::user_models::User;
    use salamandra_server::lib::db::users_db::insert_user;

    // TEST CASES
    // * Non existing user
    // * Existing user

    #[tokio::test]
    async fn test_get_user_not_found() {
        let (connector, _container) = pg_container().await;
        let user_id = Uuid::new_v4().to_string();
        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));
        
        let resp = get_user(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_user_ok() {
        let (connector, _container) = pg_container().await;
        let user = User::default();
        let user_id = user.id.to_string();
        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));
       
        let _ = insert_user(&user, &connector).await;
        let resp = get_user(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        if let Body::Text(body) = response.into_body() {
            let fetched_user: Result<User, _> = serde_json::from_str(&body);
            assert!(fetched_user.is_ok()); 
            assert_eq!(user, fetched_user.unwrap());
        }
    }
}

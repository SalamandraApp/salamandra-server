use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::users_db::search_username;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBConnector;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct UserInfo {
    username: String,
    display_name: String,
    id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct UserSearchResult {
    users: Vec<UserInfo>,
}

/// Return all users with username that matches the given prefix
pub async fn search_users(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Check query paramater
    let username = match event.query_string_parameters().first("username") {
        Some(name) => name.to_string(),
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, ""))
    };

    // Search in database
    let search_result = match search_username(&username, connector).await {
        Ok(vec) => vec,
        Err(error) => {
            error!("INTERNAL SERVER ERROR: {}", error);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };
    
    // Format and return results
    let user_info: Vec<UserInfo> = search_result.into_iter()
        .map(|user| UserInfo {
            username: user.username,
            display_name: user.display_name,
            id: user.id,
        })
    .collect();
    let result = UserSearchResult { users: user_info };
    Ok(build_resp(StatusCode::OK, result))
    
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use salamandra_server::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Invalid query
    //      * No query parameters
    //      * Other parameters
    // * Search multiple users

    #[tokio::test]
    async fn test_search_users_invalid_query() {
        let (connector, _container) = pg_container().await;
        { // ------ No query parameters
            let req_ = Request::default();

            let resp = search_users(req_, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
        { // ------ No username parameter
            let req_ = Request::default();

            let mut query_params = HashMap::new();
            query_params.insert("not_username".to_string(), "Test".to_string());
            let req = req_.with_query_string_parameters(query_params);

            let resp = search_users(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_search_users_ok() {
        let (connector, _container) = pg_container().await;
        let req_ = Request::default();

        let mut query_params = HashMap::new();
        query_params.insert("username".to_string(), "Test".to_string());
        let req = req_.with_query_string_parameters(query_params);
    
        let user_ids = insert_helper(5, Items::Users, &connector, Some("Test".into())).await;
        let resp = search_users(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        if let Body::Text(body) = response.into_body() {
            let res: Result<UserSearchResult, _> = serde_json::from_str(&body);
            assert!(res.is_ok());
            let id_vec: Vec<Uuid> = res.unwrap().users.iter().map(|ex| ex.id.clone()).collect();
            assert_eq!(user_ids, id_vec);
        }
    }
}    


use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use salamandra_server::lib::db::users_db::search_username;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBPool;

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

pub async fn search_users(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let username = match event.query_string_parameters().first("username") {
        Some(name) => name.to_string(),
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, ""))
    };

    let search_result = match search_username(&username, test_db).await {
        Ok(vec) => vec,
        Err(_) => {
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };
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

    #[tokio::test]
    async fn test_search_users_invalid_query() {
        let (pool, _container) = pg_container().await;
        { // ------ No query parameters
            let req_ = Request::default();

            let resp = search_users(req_, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
        { // ------ No username parameter
            let req_ = Request::default();

            let mut query_params = HashMap::new();
            query_params.insert("not_username".to_string(), "Test".to_string());
            let req = req_.with_query_string_parameters(query_params);

            let resp = search_users(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_search_users_ok() {
        let (pool, _container) = pg_container().await;
        let req_ = Request::default();

        let mut query_params = HashMap::new();
        query_params.insert("username".to_string(), "Test".to_string());
        let req = req_.with_query_string_parameters(query_params);
    
        let user_ids = insert_helper(5, Items::Users, pool.clone(), Some("Test".into())).await;
        let resp = search_users(req, Some(pool)).await;
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


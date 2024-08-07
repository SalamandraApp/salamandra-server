use lambda_http::{Error, Request, Response, Body, RequestExt,};
use lambda_http::http::StatusCode;
use uuid::Uuid;

use salamandra_server::lib::db::users_db::lookup_user;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::errors::DBError;


pub async fn get_user(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let user_id = match event.path_parameters().first("user_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid user_id")),
    };

    match lookup_user(user_id, test_db).await {
        Ok(user) => Ok(build_resp(StatusCode::OK, user)),
        Err(DBError::ItemNotFound(mes)) => Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(_) => Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
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

    #[tokio::test]
    async fn test_get_user_invalid_user_id() {

        let (pool, _container) = pg_container().await;
        let user_id = String::from("INVALID-UUID");
        let req = Request::default();
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));
        
        let resp = get_user(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let (pool, _container) = pg_container().await;
        let user_id = Uuid::new_v4().to_string();
        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));
        
        let resp = get_user(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_user_ok() {
        let (pool, _container) = pg_container().await;
        let user = User::default();
        let user_id = user.id.to_string();
        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("user_id".to_string(), user_id)]));
       
        let _ = insert_user(&user, Some(pool.clone())).await;
        let resp = get_user(req, Some(pool)).await;
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

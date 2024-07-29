use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::exercises_db::lookup_exercise;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::errors::DBError;

pub async fn get_exercise(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let exercise_id = match event.path_parameters().first("exercise_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid or missing exercise_id")),
    };
 
    match lookup_exercise(exercise_id, test_db).await {
        Ok(exercise) => Ok(build_resp(StatusCode::OK, exercise)),
        Err(DBError::ItemNotFound(mes)) => Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(other_error) => {
            error!("INTERNAL SERVER ERROR: {}", other_error);
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
    use salamandra_server::lib::utils::tests::{pg_container, insert_helper, Items};
    use salamandra_server::lib::models::exercise_models::Exercise;

    #[tokio::test]
    async fn test_get_exercise_invalid_exercise_id() {

        let (pool, _container) = pg_container().await;
        let exercise_id = String::from("INVALID-UUID");
        let req = Request::default();
        let req = req.with_path_parameters(HashMap::from([("exercise_id".to_string(), exercise_id)]));
        
        let resp = get_exercise(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_exercise_not_found() {
        let (pool, _container) = pg_container().await;
        let exercise_id = Uuid::new_v4().to_string();
        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("exercise_id".to_string(), exercise_id)]));
        
        let resp = get_exercise(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    #[tokio::test]
    async fn test_get_exercise_ok() {
        let (pool, _container) = pg_container().await;
        let exercise_uuid = insert_helper(1, Items::Exercises, pool.clone(), None).await[0];
        let exercise_id = exercise_uuid.to_string();

        let req = Request::default();
        let req = req.clone().with_path_parameters(HashMap::from([("exercise_id".to_string(), exercise_id)]));
       
        let resp = get_exercise(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        if let Body::Text(body) = response.into_body() {
            let fetched_exercise: Result<Exercise, _> = serde_json::from_str(&body);
            assert!(fetched_exercise.is_ok()); 
            assert_eq!(exercise_uuid, fetched_exercise.unwrap().id);
        }
    }
}

use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use salamandra_server::lib::errors::DBError;
use serde::{Deserialize, Serialize};

use salamandra_server::lib::db::exercises_db::search_exercises;
use salamandra_server::lib::utils::handlers::build_resp;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::models::exercise_models::Exercise;

#[derive(Debug, Serialize, Deserialize)]
struct ExerciseSearchResult {
    exercises: Vec<Exercise>,
}

pub async fn search_exercises_(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let name = match event.query_string_parameters().first("name") {
        Some(name) => name.to_string(),
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Incorrect query parameters"))
    };

    let search_result = match search_exercises(&name, test_db).await {
        Ok(vec) => vec,
        Err(DBError::ConnectionError(ref mes)) => return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, mes)),
        Err(_) => return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
    };
    let result = ExerciseSearchResult { exercises: search_result};
    Ok(build_resp(StatusCode::OK, result))
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    use salamandra_server::lib::utils::tests::{pg_container, insert_helper, Items};

    #[tokio::test]
    async fn test_search_exercises_invalid_query() {
        let (pool, _container) = pg_container().await;
        { // ------ No query parameters
            let req_ = Request::default();

            let resp = search_exercises_(req_, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
        { // ------ No username parameter
            let req_ = Request::default();

            let mut query_params = HashMap::new();
            query_params.insert("not_username".to_string(), "Test".to_string());
            let req = req_.with_query_string_parameters(query_params);

            let resp = search_exercises_(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_search_exercises_ok() {
        let (pool, _container) = pg_container().await;
        let req_ = Request::default();

        let mut query_params = HashMap::new();
        query_params.insert("name".to_string(), "Test".to_string());
        let req = req_.with_query_string_parameters(query_params);
    
        let exercise_ids = insert_helper(5, Items::Exercises, pool.clone(), Some("Test".into())).await;
        let resp = search_exercises_(req, Some(pool)).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        if let Body::Text(body) = response.into_body() {
            let res: Result<ExerciseSearchResult, _> = serde_json::from_str(&body);
            assert!(res.is_ok());
            let id_vec: Vec<Uuid> = res.unwrap().exercises.iter().map(|ex| ex.id.clone()).collect();
            assert_eq!(exercise_ids, id_vec);
        }
    }
}

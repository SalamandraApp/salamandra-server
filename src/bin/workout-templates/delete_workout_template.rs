use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::workout_templates_db::delete_workout_template;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBPool;


pub async fn delete_workout_template_(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let user_id = match event.path_parameters().first("user_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid user_id")),
    };
    
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };

    let workout_template_id = match event.path_parameters().first("workout_template_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid workout_template_id")),
    };


    match delete_workout_template(user_id, workout_template_id, test_db).await {
        Ok(deleted) => {
            if deleted > 0 {
                return Ok(build_resp(StatusCode::NO_CONTENT, ""));
            }
            Ok(build_resp(StatusCode::NOT_FOUND, ""))
        }
        Err(mes) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use lambda_http::http::header::{AUTHORIZATION, HeaderValue};
    use salamandra_server::lib::utils::tests::{pg_container, test_jwt, insert_helper, Items};
    use salamandra_server::lib::db::workout_templates_db::lookup_workout_template;

    #[tokio::test]
    async fn test_delete_workout_template_invalid_ids() {
        let (pool, _container) = pg_container().await;

        { // ------ Invalid uuid format
            let user_id = Uuid::new_v4();
            let user_id_string = user_id.to_string();
            let workout_template_id_string = String::from("INVALID");
            let mut req = Request::default();
            let jwt = test_jwt(user_id);

            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


            let resp = delete_workout_template_(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        { // ------ Missing fields
            let user_id = Uuid::new_v4();
            let user_id_string = user_id.to_string();
            let mut req = Request::default();
            let jwt = test_jwt(user_id);

            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id_string)])
            );
            let resp = delete_workout_template_(req, Some(pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

    }

    #[tokio::test]
    async fn test_delete_workout_template_none() {
        let (pool, _container) = pg_container().await;

        let user_id = insert_helper(1, Items::Users, pool.clone(), None).await[0];
        let user_id_string = user_id.to_string();
        let workout_template_id = Uuid::new_v4();
        let workout_template_id_string = workout_template_id.to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


        let resp = delete_workout_template_(req, Some(pool.clone())).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_workout_template_success() {
        let (pool, _container) = pg_container().await;
        let workout_template_id = insert_helper(1, Items::WkTemplates, pool.clone(), None).await[0];
        let user_id = lookup_workout_template(workout_template_id, Some(pool.clone())).await.unwrap().user_id;
        let user_id_string = user_id.to_string();
        let workout_template_id_string = workout_template_id.to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


        let resp = delete_workout_template_(req, Some(pool.clone())).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

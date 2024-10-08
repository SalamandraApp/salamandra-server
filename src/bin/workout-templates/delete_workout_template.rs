use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::workout_templates_db::delete_workout_template;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBConnector;


/// Remove template from database
pub async fn delete_workout_template_(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();
    let workout_template_id = Uuid::parse_str(event.path_parameters().first("workout_template_id").unwrap()).unwrap();
   
    // Check user in claim
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };

    // Delete in database
    match delete_workout_template(user_id, workout_template_id, connector).await {
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

    // TEST CASES
    // * Non existing template
    // * Success

    #[tokio::test]
    async fn test_delete_workout_template_none() {
        let (connector, _container) = pg_container().await;

        let user_id = insert_helper(1, Items::Users, &connector, None).await[0];
        let user_id_string = user_id.to_string();
        let workout_template_id = Uuid::new_v4();
        let workout_template_id_string = workout_template_id.to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


        let resp = delete_workout_template_(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_workout_template_success() {
        let (connector, _container) = pg_container().await;
        let workout_template_id = insert_helper(1, Items::WkTemplates, &connector, None).await[0];
        let user_id = lookup_workout_template(workout_template_id, &connector).await.unwrap().user_id;
        let user_id_string = user_id.to_string();
        let workout_template_id_string = workout_template_id.to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


        let resp = delete_workout_template_(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

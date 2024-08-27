use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use salamandra_server::lib::models::workout_execution_models::{WkExecutionWithElements, WorkoutExecutionFull};
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::workout_executions_db::lookup_workout_execution_validated;
use salamandra_server::lib::db::wk_execution_elements_db::{select_wk_execution_element_by_execution, select_wk_execution_element_by_execution_full};
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::errors::DBError;


/// Fetch execution with specified verbosity
/// * Assumes path parameters have been checked previously
pub async fn get_workout_execution(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {
    
    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();
    let execution_id = Uuid::parse_str(event.path_parameters().first("workout_execution_id").unwrap()).unwrap();

    // Confirm user making call owns resource
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };
    
    // Check verbosity of response
    let full: bool = match event.query_string_parameters().first("full") {
        Some(val) => val == "true",
        None => false
    };

    // Get user from execution id
    let workout_execution = match lookup_workout_execution_validated(execution_id, user_id, connector).await  {
        Ok(execution) => execution,
        Err(DBError::ItemNotFound(mes)) => return Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(mes) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            println!("MES: {}", mes);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };
   
    // Fetch full/normal elements and build response
    match full {
        true => {
        let full_elements = match select_wk_execution_element_by_execution_full(execution_id, connector).await {
            Ok(vector) => vector,
            Err(mes) => {
                error!("INTERNAL SERVER ERROR: {}", mes);
                return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
            }
        };

        let execution = WorkoutExecutionFull {
            workout_execution,
            elements: full_elements,
        };
        Ok(build_resp(StatusCode::OK, execution))
    },
        false => {
            let elements = match select_wk_execution_element_by_execution(execution_id, connector).await {
                Ok(vector) => vector,
                Err(mes) => {
                    error!("INTERNAL SERVER ERROR: {}", mes);
                    return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
                }
            };

            let execution = WkExecutionWithElements {
                workout_execution,
                elements,
            };
            Ok(build_resp(StatusCode::OK, execution))
        },
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use lambda_http::http::header::{AUTHORIZATION, HeaderValue};
    use salamandra_server::lib::db::workout_templates_db::lookup_workout_template;
    use salamandra_server::lib::utils::tests::{pg_container, test_jwt, insert_helper, Items};
    use salamandra_server::lib::db::wk_execution_elements_db::lookup_wk_execution_element;
    use salamandra_server::lib::db::workout_executions_db::lookup_workout_execution;

    // TEST CASES
    // * Get non exisiting execution
    // * Success full and non full

    #[tokio::test]
    async fn test_get_workout_execution_not_found() {
        let (connector, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let user_id_string = user_id.to_string();
        let workout_execution_id_string = Uuid::new_v4().to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_execution_id".into(), workout_execution_id_string)]));


        let resp = get_workout_execution(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    #[tokio::test]
    async fn test_get_workout_execution_success_full() {
        let (connector, _container) = pg_container().await;
       
        let element_vector = insert_helper(5, Items::WkExecutionElements, &connector, None).await;
        let execution_id = lookup_wk_execution_element(element_vector[0], &connector).await.unwrap().workout_execution_id;
        let template_id = lookup_workout_execution(execution_id, &connector).await.unwrap().workout_template_id;
        let user_id = lookup_workout_template(template_id, &connector).await.unwrap().user_id;

        let user_id_string = user_id.to_string();
        let jwt = test_jwt(user_id);
        
        let req = Request::default();
        
        let mut query_params = HashMap::new();
        query_params.insert("full".to_string(), "true".to_string());
        let mut req = req.with_query_string_parameters(query_params);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_execution_id".into(), execution_id.to_string())]));


        let resp = get_workout_execution(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        if let Body::Text(body) = response.into_body() {
            let execution: Result<WorkoutExecutionFull, _> = serde_json::from_str(&body);
            let id_vec: Vec<Uuid> = execution.unwrap().elements.iter().map(|wkt| wkt.id.clone()).collect();
            assert_eq!(id_vec.len(), 5);
            assert_eq!(id_vec, element_vector);
        }
    }
    
    #[tokio::test]
    async fn test_get_workout_execution_success_not_full() {
        let (connector, _container) = pg_container().await;
       
        let element_vector = insert_helper(5, Items::WkExecutionElements, &connector, None).await;
        let execution_id = lookup_wk_execution_element(element_vector[0], &connector).await.unwrap().workout_execution_id;
        let template_id = lookup_workout_execution(execution_id, &connector).await.unwrap().workout_template_id;
        let user_id = lookup_workout_template(template_id, &connector).await.unwrap().user_id;

        let user_id_string = user_id.to_string();
        let jwt = test_jwt(user_id);
        
        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_execution_id".into(), execution_id.to_string())]));


        let resp = get_workout_execution(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        if let Body::Text(body) = response.into_body() {
            let execution: Result<WkExecutionWithElements, _> = serde_json::from_str(&body);
            let id_vec: Vec<Uuid> = execution.unwrap().elements.iter().map(|wkt| wkt.id.clone()).collect();
            assert_eq!(id_vec.len(), 5);
            assert_eq!(id_vec, element_vector);
        }
    }

}

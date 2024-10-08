use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use salamandra_server::lib::models::workout_templates_models::{WkTemplateWithElements, WorkoutTemplateFull};
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::workout_templates_db::lookup_workout_template;
use salamandra_server::lib::db::wk_template_elements_db::{select_wk_template_element_by_template, select_wk_template_element_by_template_full};
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::errors::DBError;


/// Fetch template from a given user
pub async fn get_workout_template(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();
    let workout_template_id = Uuid::parse_str(event.path_parameters().first("workout_template_id").unwrap()).unwrap();
    
    // Check claim
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };
   
    // Get query parameter
    let full: bool = match event.query_string_parameters().first("full") {
        Some(val) => val == "true",
        None => false
    };

    let workout_template = match lookup_workout_template(workout_template_id, connector).await  {
        Ok(template) => {
            if template.user_id != user_id {
                return Ok(build_resp(StatusCode::NOT_FOUND, "No template exists with the corresponding id"));
            }
            template
        }
        Err(DBError::ItemNotFound(mes)) => return Ok(build_resp(StatusCode::NOT_FOUND, mes)),
        Err(mes) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };

    // Select template and build response
    match full {
        true => {
        let full_elements = match select_wk_template_element_by_template_full(workout_template_id, connector).await {
            Ok(vector) => vector,
            Err(mes) => {
                error!("INTERNAL SERVER ERROR: {}", mes);
                return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
            }
        };

        // Create models 
        let template = WorkoutTemplateFull {
            workout_template,
            elements: full_elements,
        };
        Ok(build_resp(StatusCode::OK, template))
    },
        false => {
            let elements = match select_wk_template_element_by_template(workout_template_id, connector).await {
                Ok(vector) => vector,
                Err(mes) => {
                    error!("INTERNAL SERVER ERROR: {}", mes);
                    return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
                }
            };

            // Create models 
            let template = WkTemplateWithElements {
                workout_template,
                elements,
            };
            Ok(build_resp(StatusCode::OK, template))
        },
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use lambda_http::http::header::{AUTHORIZATION, HeaderValue};
    use salamandra_server::lib::utils::tests::{pg_container, test_jwt, insert_helper, Items};
    use salamandra_server::lib::db::wk_template_elements_db::lookup_wk_template_element;
    use salamandra_server::lib::db::workout_templates_db::lookup_workout_template;

    // TEST CASES
    // * Non existing template
    // * Success full and not full
    
    #[tokio::test]
    async fn test_get_workout_template_not_found() {
        let (connector, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let user_id_string = user_id.to_string();
        let workout_template_id_string = Uuid::new_v4().to_string();
        let mut req = Request::default();
        let jwt = test_jwt(user_id);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), workout_template_id_string)]));


        let resp = get_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
    
    #[tokio::test]
    async fn test_get_workout_template_success_full() {
        let (connector, _container) = pg_container().await;
       
        let element_vector = insert_helper(5, Items::WkTemplateElements, &connector, None).await;
        let template_id = lookup_wk_template_element(element_vector[0], &connector).await.unwrap().workout_template_id;
        let user_id = lookup_workout_template(template_id, &connector).await.unwrap().user_id;

        let user_id_string = user_id.to_string();
        let jwt = test_jwt(user_id);
        
        let req = Request::default();
        
        let mut query_params = HashMap::new();
        query_params.insert("full".to_string(), "true".to_string());
        let mut req = req.with_query_string_parameters(query_params);

        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), template_id.to_string())]));


        let resp = get_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        if let Body::Text(body) = response.into_body() {
            let template: Result<WorkoutTemplateFull, _> = serde_json::from_str(&body);
            let id_vec: Vec<Uuid> = template.unwrap().elements.iter().map(|wkt| wkt.id.clone()).collect();
            assert_eq!(id_vec.len(), 5);
            assert_eq!(id_vec, element_vector);
        }
    }
    
    #[tokio::test]
    async fn test_get_workout_template_success_not_full() {
        let (connector, _container) = pg_container().await;
       
        let element_vector = insert_helper(5, Items::WkTemplateElements, &connector, None).await;
        let template_id = lookup_wk_template_element(element_vector[0], &connector).await.unwrap().workout_template_id;
        let user_id = lookup_workout_template(template_id, &connector).await.unwrap().user_id;

        let user_id_string = user_id.to_string();
        let jwt = test_jwt(user_id);
        
        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string), ("workout_template_id".into(), template_id.to_string())]));


        let resp = get_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        if let Body::Text(body) = response.into_body() {
            let template: Result<WkTemplateWithElements, _> = serde_json::from_str(&body);
            let id_vec: Vec<Uuid> = template.unwrap().elements.iter().map(|wkt| wkt.id.clone()).collect();
            assert_eq!(id_vec.len(), 5);
            assert_eq!(id_vec, element_vector);
        }
    }

}

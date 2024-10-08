use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use salamandra_server::lib::db::workout_templates_db::select_workout_template_by_user;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::models::workout_templates_models::WorkoutTemplate;

#[derive(Serialize, Deserialize)]
struct GetAllTemplatesResponse {
    count: usize,
    templates: Vec<WorkoutTemplate>
}

/// Fetch all templates for a given
pub async fn get_all_workout_templates(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();
    
    // Check user in claim
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };
    
    // Select from database and prepare response
    match select_workout_template_by_user(user_id, connector).await {
        Ok(vec) => {
            let response = GetAllTemplatesResponse {
                count: vec.len(),
                templates: vec,
            };
            Ok(build_resp(StatusCode::OK, response))

        },
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
    // * Success

    #[tokio::test]
    async fn test_get_all_workout_multiple() {
        let (connector, _container) = pg_container().await;
        
        let wk_templates = insert_helper(5, Items::WkTemplates, &connector, None).await;
        let user_id = lookup_workout_template(wk_templates[0], &connector).await.unwrap().user_id;
        let user_id_string = user_id.to_string();
        let jwt = test_jwt(user_id);
        
        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(HashMap::from([("user_id".to_string(), user_id_string)]));

        let resp = get_all_workout_templates(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        if let Body::Text(body) = response.into_body() {
            let templates: Result<GetAllTemplatesResponse, _> = serde_json::from_str(&body);
            let id_vec: Vec<Uuid> = templates.unwrap().templates.iter().map(|wkt| wkt.id.clone()).collect();
            assert_eq!(id_vec.len(), 5);
            assert_eq!(id_vec, wk_templates);
        }
        
    }
}

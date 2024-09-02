use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

use salamandra_server::lib::db::exercises_db::validate_exercises;
use salamandra_server::lib::db::workout_templates_db::{insert_workout_template, delete_workout_template};
use salamandra_server::lib::db::wk_template_elements_db::insert_batch_wk_template_elements;
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::models::workout_templates_models::{NewWorkoutTemplate, WkTemplateWithElements, WorkoutTemplate};
use salamandra_server::lib::models::wk_template_elements_models::NewWkTemplateElement;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::errors::DBError;

const BASE_ERROR: &str = "Invalid payload. ";
const DOC_LINK: &str = ". See https://github.com/SalamandraApp/salamandra-server/wiki/Workout-executions-API#createwkexecutionrequest for the correct format. If you think its an error leave an issue in the repository.";

#[derive(Serialize, Deserialize)]
struct CreateWkTemplateRequest {
    name: String,
    description: Option<String>,
    date_created: chrono::NaiveDate,
    elements: Vec<WkTemplateElementRequest>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WkTemplateElementRequest {
    exercise_id: Uuid,
    position: i16,
    reps: i16,
    sets: i16,
    weight: Option<f32>,
    rest: i16,
    super_set: Option<i16>,
}

impl CreateWkTemplateRequest {
    fn to_new_template(&self, user_id: Uuid) -> NewWorkoutTemplate {
        NewWorkoutTemplate {
            user_id,
            name: self.name.clone(),
            description: self.description.clone(),
            date_created: self.date_created,
        }
    }
}
impl WkTemplateElementRequest {
    fn to_new_element(&self, workout_template_id: Uuid) -> NewWkTemplateElement {
        NewWkTemplateElement {
            workout_template_id,
            exercise_id: self.exercise_id,
            position: self.position,
            reps: self.reps,
            sets: self.sets,
            weight: self.weight,
            rest: self.rest,
            super_set: self.super_set,
        }
    }
}

/// Insert new workout template and its elements
pub async fn create_workout_template(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {

    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();

    // Check user is the same as sub in claim
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };

    // Check and extract payload
    let body = match event.into_body() {
        Body::Text(body) => body,
        _ => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload")),
    };
    let req: CreateWkTemplateRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(_) => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload")),
    };

    // Check input
    if chrono::Utc::now().date_naive() < req.date_created {
        return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload: date can't be in the future"));
    }

    // Validate elements
    match validate_template(&req.elements){
        Ok(_) => (),
        Err(error_message) => return Ok(build_resp(StatusCode::BAD_REQUEST, error_message))
    };

    // Validate that the ids exist
    let exercise_ids: HashSet<Uuid> = req.elements.iter().map(|element| element.exercise_id).collect();
    match validate_exercises(exercise_ids.into_iter().collect(), connector).await {
        Ok(valid) => {
            if !valid {
                return Ok(build_resp(StatusCode::NOT_FOUND, "One or more exercise IDs do not reference existing exercises"));
            }
        }
        Err(error) => {
            error!("INTERNAL SERVER ERROR: {}", error);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }

    // Construct and insert template
    let new_workout_template = req.to_new_template(user_id);

    // Insert template
    let workout_template: WorkoutTemplate = match insert_workout_template(&new_workout_template, connector).await {
        Ok(template) => template,
        Err(DBError::UniqueViolation(mes)) => {
            // should never trigger since the primary key is only the UUID
            warn!("Tried to insert already exisiting workout-template");
            return Ok(build_resp(StatusCode::CONFLICT, mes))
        }
        Err(mes) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };

    // Create template elements
    let new_elements: Vec<NewWkTemplateElement> = req.elements
        .iter()
        .map(|elem| elem.to_new_element(workout_template.id))
        .collect();

    // Insert template elements
    match insert_batch_wk_template_elements(&new_elements, connector).await {
        Ok(elements) => {
            let response = WkTemplateWithElements {
                workout_template,
                elements,
            };
            Ok(build_resp(StatusCode::CREATED, response))
        },
        Err(DBError::ConnectionError(mes)) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
        Err(error) => {
            // Should never trigger because all values are checked before hand
            warn!("Could not insert workout-template element: {}", error);
            let result_delete = delete_workout_template(user_id, workout_template.id, connector).await;
            if result_delete.is_err() {
                warn!("Could not delete workout-template triggered by error inserting templates");
            }
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}

/// Check request before inserting
/// * All values >= 0
/// * Sets, reps, weight > 0
/// * Position sequential from 1
/// * Superset sequential from 1 with repeats, at least 2 repeats for each non null value
/// * Positions in each non null superset value must be sequential
fn validate_template(items: &[WkTemplateElementRequest]) -> Result<(), String> {
    if items.is_empty() {
        return Err(format!("{}There must be at least one element in the workout template{}", BASE_ERROR, DOC_LINK));
    }
   
    // Sets and reps over 0
    if items.iter().any(|item| item.sets <= 0 || item.reps <= 0 || item.rest < 0 || item.weight.map_or(false, |w| w < 0.0)) {
        return Err(format!("{}All sets and reps must be at least 1. No values can't be negative{}", BASE_ERROR, DOC_LINK));
    }

    // Sequential position 
    let mut positions: Vec<i16> = items.iter().map(|item| item.position).collect();
    positions.sort_unstable();
    if positions != (1..=items.len() as i16).collect::<Vec<_>>() { 
        return Err(format!("{}The element's positions must be sequential, starting from 1{}", BASE_ERROR, DOC_LINK));
    }

    // Group by superset
    let mut super_set_map: HashMap<Option<i16>, Vec<&WkTemplateElementRequest>> = HashMap::new();
    for item in items {
        super_set_map.entry(item.super_set).or_insert_with(Vec::new).push(item);
    }

    // SUPER SET
    let mut super_set_map: HashMap<Option<i16>, Vec<&WkTemplateElementRequest>> = HashMap::new();
    for item in items {
        super_set_map.entry(item.super_set).or_insert_with(Vec::new).push(item);
    }
    let mut unique_super_set_values: Vec<i16> = super_set_map.keys()
        .filter_map(|&k| k)  
        .collect::<HashSet<i16>>()  
        .into_iter()
        .collect();
    unique_super_set_values.sort_unstable();

    if !unique_super_set_values.is_empty() && 
        (unique_super_set_values[0] != 1 || 
         unique_super_set_values.windows(2).any(|w| w[1] - w[0] > 1)) {
            return Err(format!("{}The non-null super_set values must be sequential, starting from 1 with repetitions{}", BASE_ERROR, DOC_LINK));
    }

    // Super set group
    for (super_set, group) in super_set_map {
        if let Some(_) = super_set {
            // At least 2 exercises
            if group.len() < 2 {
                return Err(format!("{}There must be at least 2 elements per super set group{}", BASE_ERROR, DOC_LINK));
            }

            // Sequential positions within superset
            let mut ss_positions: Vec<i16> = group.iter().map(|item| item.position).collect();
            ss_positions.sort_unstable();
            let expected_ss_positions: Vec<i16> = (ss_positions[0]..=ss_positions[0] + group.len() as i16 - 1)
                .collect();
            if ss_positions != expected_ss_positions {
                return Err(format!("{}In each super_set group all position values must be sequential{}", BASE_ERROR, DOC_LINK));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;
    use lambda_http::http::header::{AUTHORIZATION, HeaderValue};
    use salamandra_server::lib::utils::tests::{insert_helper, pg_container, test_jwt, Items};

    // TEST CASES
    // * Create a template
    // * Invalid ids
    // * Invalid payload

    async fn setup_template(connector: &DBConnector) -> (Uuid, CreateWkTemplateRequest) {
        let user_id = insert_helper(1, Items::Users, connector, None).await[0];
        let exercise_id = insert_helper(1, Items::Exercises, connector, None).await[0];

        let base_element = WkTemplateElementRequest {
            exercise_id,
            position: 1,  // We'll update this later
            reps: 1,
            sets: 1,
            weight: Some(1.0),
            super_set: None,
            rest: 0,
        };

        let mut elements: Vec<WkTemplateElementRequest> = vec![base_element.clone(); 4];

        // Update positions
        for (index, element) in elements.iter_mut().enumerate() {
            element.position = (index + 1) as i16;
        }

        let template = CreateWkTemplateRequest {
            name: "Placeholder".to_string(),
            description: None,
            date_created: chrono::Utc::now().date_naive(),
            elements,
        };
        (user_id, template)
    }   

    #[tokio::test]
    async fn test_create_workout_template_success() {
        let (connector, _container) = pg_container().await;
        let (user_id, mut payload) = setup_template(&connector).await;
        let jwt = test_jwt(user_id);

        payload.elements[0].super_set = Some(1);
        payload.elements[1].super_set = Some(1);
        payload.elements[2].super_set = Some(2);
        payload.elements[3].super_set = Some(2);

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

        let resp = create_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        if let Body::Text(body) = response.into_body() {
            let template: Result<WkTemplateWithElements, _> = serde_json::from_str(&body);
            assert!(template.is_ok()); 
            assert_eq!(template.unwrap().workout_template.name, "Placeholder".to_string());
        }

    }

    #[tokio::test]
    async fn test_create_workout_template_invalid_user_ids() {
        let (connector, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let jwt = test_jwt(user_id);

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

        let resp = create_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    }


    #[derive(Serialize, Deserialize)]
    struct BadPayload {
        test: i32
    }
    #[tokio::test]
    async fn test_create_workout_template_invalid_payload() {

        let (connector, _container) = pg_container().await;
        let (user_id, _payload) = setup_template(&connector).await;
        let payload = BadPayload {test: 1};
        let jwt = test_jwt(user_id);

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
        );

        let resp = create_workout_template(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_workout_template_invalid_fields() {
        let (connector, _container) = pg_container().await;
        { // ------ Invalid date
            let (user_id, mut payload) = setup_template(&connector).await;
            let jwt = test_jwt(user_id);
            payload.date_created = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap()
                .date_naive();

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
                );

            let resp = create_workout_template(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, "Invalid payload: date can't be in the future");
            }
        }

        { // ------ Wrong exercise ids 
            let (user_id, mut payload) = setup_template(&connector).await;
            let jwt = test_jwt(user_id);
            payload.elements[0].exercise_id = Uuid::new_v4();

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
                );

            let resp = create_workout_template(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);

        }
        { // Negative values
            let (connector, _container) = pg_container().await;
            let (user_id1, mut payload1) = setup_template(&connector).await;

            payload1.elements[0].reps = 0;
            payload1.elements[0].rest = -1;


            let jwt1 = test_jwt(user_id1);

            let payloads = vec![
                (user_id1, jwt1, payload1),
            ];

            for (user_id, jwt, payload) in payloads {
                let mut req = Request::default();
                req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
                *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
                let req = req.with_path_parameters(
                    HashMap::from([("user_id".to_string(), user_id.to_string())])
                );

                let resp = create_workout_template(req, &connector).await;
                assert!(resp.is_ok());
                let response = resp.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
                if let Body::Text(body) = response.into_body() {
                    let body_string: String = body;
                    let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                    assert_eq!(unescaped_body, format!("{}All sets and reps must be at least 1. No values can't be negative{}", BASE_ERROR, DOC_LINK));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_create_workout_template_invalid_position() {
        let (connector, _container) = pg_container().await;
        let (user_id1, mut payload1) = setup_template(&connector).await;
        let (user_id2, mut payload2) = setup_template(&connector).await;
        let (user_id3, mut payload3) = setup_template(&connector).await;

        payload1.elements[0].position = 0;
        payload1.elements[1].position = 1;
        payload1.elements[2].position = 1;
        payload1.elements[3].position = 2;

        payload2.elements[0].position = 1;
        payload2.elements[1].position = 2;
        payload2.elements[2].position = 8;
        payload2.elements[3].position = 3;

        payload3.elements[0].position = 0;
        payload3.elements[1].position = 1;
        payload3.elements[2].position = 3;
        payload3.elements[3].position = 4;

        let jwt1 = test_jwt(user_id1);
        let jwt2 = test_jwt(user_id2);
        let jwt3 = test_jwt(user_id3);

        let payloads = vec![
            (user_id1, jwt1, payload1),
            (user_id2, jwt2, payload2),
            (user_id3, jwt3, payload3),
        ];

        for (user_id, jwt, payload) in payloads {
            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_template(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}The element's positions must be sequential, starting from 1{}", BASE_ERROR, DOC_LINK));
            }
        }
    }

    #[tokio::test]
    async fn test_create_workout_template_invalid_super_set() {
        let (connector, _container) = pg_container().await;
        let (user_id1, mut payload1) = setup_template(&connector).await;
        let (user_id2, mut payload2) = setup_template(&connector).await;
        let (user_id3, mut payload3) = setup_template(&connector).await;

        // Not sequential super set id
        payload1.elements[0].super_set = Some(1);
        payload1.elements[1].super_set = Some(1);
        payload1.elements[2].super_set = Some(3);
        payload1.elements[3].super_set = Some(3);

        // Only 1 exercise in superset
        payload2.elements[0].super_set = Some(1);

        // Not sequential in superset
        payload3.elements[0].super_set = Some(1);
        payload3.elements[2].super_set = Some(1);

        let jwt1 = test_jwt(user_id1);
        let jwt2 = test_jwt(user_id2);
        let jwt3 = test_jwt(user_id3);

        let payloads = vec![
            (user_id1, jwt1, payload1),
            (user_id2, jwt2, payload2),
            (user_id3, jwt3, payload3),
        ];
        let responses = vec![
            format!("{}The non-null super_set values must be sequential, starting from 1 with repetitions{}", BASE_ERROR, DOC_LINK),
            format!("{}There must be at least 2 elements per super set group{}", BASE_ERROR, DOC_LINK),
            format!("{}In each super_set group all position values must be sequential{}", BASE_ERROR, DOC_LINK)
        ];

        for (index, (user_id, jwt, payload)) in payloads.iter().enumerate() {
            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error serializing payload"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_template(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, responses[index]);
            }

        }
    }
}

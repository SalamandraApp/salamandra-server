use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;
use itertools::Itertools;
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};

use salamandra_server::lib::db::exercises_db::validate_exercises;
use salamandra_server::lib::db::DBConnector;
use salamandra_server::lib::db::wk_execution_elements_db::insert_batch_wk_execution_elements;
use salamandra_server::lib::db::workout_executions_db::{delete_workout_execution, insert_workout_execution};
use salamandra_server::lib::models::wk_execution_elements_models::NewWkExecutionElement;
use salamandra_server::lib::models::workout_execution_models::{NewWorkoutExecution, WkExecutionWithElements, WorkoutExecution};
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::errors::DBError;

const BASE_ERROR: &str = "Invalid payload. ";
const DOC_LINK: &str = ". See https://github.com/SalamandraApp/salamandra-server/wiki/Workout-executions-API#createwkexecutionrequest for the correct format. If you think its an error leave an issue in the repository.";

#[derive(Serialize, Deserialize)]
struct CreateWkExecutionRequest {
    workout_template_id: Uuid,
    date: NaiveDate,
    survey: i16,
    elements: Vec<WkExecutionElementRequest>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WkExecutionElementRequest {
    exercise_id: Uuid,
    position: i16,
    exercise_number: i16,
    reps: i16,
    set_number: i16,
    weight: Option<f32>,
    rest: i16,
    super_set: Option<i16>,
    time: i32,
}

impl CreateWkExecutionRequest {
    fn to_new_execution(&self) -> NewWorkoutExecution {
        NewWorkoutExecution {
            workout_template_id: self.workout_template_id,
            survey: self.survey as i16,
            date: self.date,
        }
    }
}
impl WkExecutionElementRequest {
    fn to_new_element(&self, workout_execution_id: Uuid) -> NewWkExecutionElement {
        NewWkExecutionElement {
            workout_execution_id,
            exercise_id: self.exercise_id,
            position: self.position,
            exercise_number: self.exercise_number,
            reps: self.reps,
            set_number: self.set_number,
            weight: self.weight,
            rest: self.rest,
            super_set: self.super_set,
            time: self.time,
        }    
    }
}


/// Validate and insert execution
/// * Assumes path parameters have been checked previously
/// * Check all payload values
pub async fn create_workout_execution(event: Request, connector: &DBConnector) -> Result<Response<Body>, Error> {
   
    // Get path parameter
    let user_id = Uuid::parse_str(event.path_parameters().first("user_id").unwrap()).unwrap();

    // Check path user id with sub in claim
    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };

    // Check and extract payload
    let body = match event.into_body() {
        Body::Text(body) => body,
        _ => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload: can't extract body")),
    };
    let req: CreateWkExecutionRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(_) => return Ok(build_resp(StatusCode::BAD_REQUEST, format!("{}Body doesn't match request{}", BASE_ERROR, DOC_LINK))),
    };

    // Check input
    if chrono::Utc::now().date_naive() < req.date {
        return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload: date can't be in the future"));
    }

    // Elements validation
    match validate_execution(&req.elements) {
        Ok(_) => (),
        Err(error_message) => return Ok(build_resp(StatusCode::BAD_REQUEST, error_message))
    }

    // Validate exercise ids before inserting
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

    // Construct and insert execution without elements
    let new_workout_execution = req.to_new_execution();

    let workout_execution: WorkoutExecution = match insert_workout_execution(&new_workout_execution, connector).await {
        Ok(execution) => execution,
        Err(DBError::UniqueViolation(mes)) => {
            // should never trigger since the primary key is only the UUID
            warn!("Tried to insert already exisiting workout-execution");
            return Ok(build_resp(StatusCode::CONFLICT, mes))
        }
        Err(mes) => {
            error!("INTERNAL SERVER ERROR: {}", mes);
            return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    };

    // Create execution elements
    let new_elements: Vec<NewWkExecutionElement> = req.elements
        .iter()
        .map(|elem| elem.to_new_element(workout_execution.id))
        .collect();

    // Insert template elements
    match insert_batch_wk_execution_elements(&new_elements, connector).await {
        Ok(elements) => {
            let response = WkExecutionWithElements {
                workout_execution,
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
            warn!("Could not insert workout-execution element: {}", error);
            let result_delete = delete_workout_execution(workout_execution.id, connector).await;
            if result_delete.is_err() {
                warn!("Could not delete workout-execution triggered by error inserting templates");
            }
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}

/// Check format
fn validate_execution(items: &[WkExecutionElementRequest]) -> Result<(), String> {
    if items.is_empty() {
        return Err(format!("{}There must be at least one element in the workout execution{}", BASE_ERROR, DOC_LINK));
    }

    // Values over 0
    if items.iter().any(|item| item.reps <= 0 || item.exercise_number <= 0 || item.set_number <= 0 || item.weight.map_or(false, |w| w < 0.0) || item.rest < 0 || item.time <= 0) {
        return Err(format!("{}All reps, set_number and time values must be at least 1. No values can't be negative{}", BASE_ERROR, DOC_LINK));
    }

    // POSITION
    let mut positions: Vec<i16> = items.iter().map(|item| item.position).collect();
    positions.sort_unstable();
    if positions != (1..=items.len() as i16).collect::<Vec<_>>() { 
        return Err(format!("{}The element's positions must be sequential, starting from 1{}", BASE_ERROR, DOC_LINK));
    }

    // EXERCISE NUMBER
    let mut unique_exercise_numbers: Vec<i16> = items.iter()
        .map(|item| item.exercise_number)
        .collect::<HashSet<i16>>()  
        .into_iter()
        .collect();
    unique_exercise_numbers.sort_unstable();

    if unique_exercise_numbers.is_empty() || 
        unique_exercise_numbers[0] != 1 || 
            unique_exercise_numbers.windows(2).any(|w| w[1] - w[0] > 1) {
                return Err(format!("{}The element's exercise_number must be sequential, starting from 1 (repetitions allowed){}", BASE_ERROR, DOC_LINK));
    }

    // SET NUMBER
    // * In a given superset, when sorted by position, set number should never decrease
    // * For each exercise_number value, set number should be sequential without repetitions
    let invalid_set = items.iter()
        .filter_map(|item| item.super_set.map(|_| item))
        .into_group_map_by(|item| item.super_set)
        .into_iter()
        .any(|(_, group)| {
            let mut prev_set_number = 0; // less than minimum
            !group
                .iter().sorted_by_key(|item| item.position)
                .all(|item| {
                    let is_valid = item.set_number >= prev_set_number;
                    prev_set_number = item.set_number;
                    is_valid
                })
        });
    if invalid_set {
        return Err(format!("{}The set_number can't decrease for rows in a superset, when sorted by position{}", BASE_ERROR, DOC_LINK));
    }
    let invalid_exercise = items.iter()
        .into_group_map_by(|item| item.exercise_number)
        .into_iter()
        .any(|(_, group)| {
            let sorted_group = group.iter().sorted_by_key(|item| item.position);
            let set_numbers: Vec<i16> = sorted_group.map(|item| item.set_number).collect();
            let expected: Vec<i16> = (1..=set_numbers.len() as i16).collect();
            set_numbers != expected
        });
    if invalid_exercise {
        return Err(format!("{}The set_number is strictly sequential for all exercise number values, when sorted by position{}", BASE_ERROR, DOC_LINK));
    }

    // SUPER SET
    let mut super_set_map: HashMap<Option<i16>, Vec<&WkExecutionElementRequest>> = HashMap::new();
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
            // At least 2 different exercise numbers
            let distinct_exercises: HashSet<i16> = group.iter().map(|item| item.exercise_number).collect();
            if distinct_exercises.len() < 2 {
                return Err(format!("{}There must be at least 2 different exercise numbers per super_set group{}", BASE_ERROR, DOC_LINK));
            }
            // Sequential positions within superset
            let mut ss_positions: Vec<i16> = group.iter().map(|item| item.position).collect();
            ss_positions.sort_unstable();
            if ss_positions != (ss_positions[0]..=ss_positions[0] + group.len() as i16 - 1).collect::<Vec<i16>>() {
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
    use salamandra_server::lib::{db::workout_templates_db::lookup_workout_template, utils::tests::{insert_helper, pg_container, test_jwt, Items}};

    // TEST CASES
    // * Create a template
    // * Invalid ids
    // * Invalid payload
    //      * Set number
    //      * Exercise number (and negative values)
    //      * Superset
    
    async fn setup_execution(connector: &DBConnector, n: usize) -> (Uuid, CreateWkExecutionRequest) {
        let exercise_id = insert_helper(1, Items::Exercises, connector, None).await[0];
        let workout_template_id = insert_helper(1, Items::WkTemplates, connector, None).await[0];
        let user_id = lookup_workout_template(workout_template_id, connector).await.unwrap().user_id;
        
        let base_element = WkExecutionElementRequest {
            exercise_id,
            position: 0,
            set_number: 1,
            exercise_number: 1,
            reps: 1,
            weight: Some(1.0),
            rest: 0,
            super_set: None,
            time: 10,
        };

        let mut elements: Vec<WkExecutionElementRequest> = vec![base_element.clone(); n];

        // Update positions
        for (index, element) in elements.iter_mut().enumerate() {
            element.position = (index + 1) as i16;
        };

        let execution = CreateWkExecutionRequest {
            workout_template_id,
            date: chrono::Utc::now().date_naive(),
            survey: 0,
            elements,
        };
        (user_id, execution)
    }

    #[tokio::test]
    async fn test_create_workout_execution_success() {
        let (connector, _container) = pg_container().await;
        let (user_id, mut payload) = setup_execution(&connector, 8).await;
        let jwt = test_jwt(user_id);

        // 3 x exercise 1
        payload.elements[0].exercise_number = 1;
        payload.elements[1].exercise_number = 1;
        payload.elements[2].exercise_number = 1;
        payload.elements[0].set_number = 1;
        payload.elements[1].set_number = 2;
        payload.elements[2].set_number = 3;


        // Super set {
        //      3 x exercise 2
        //      2 x exercise 3
        // }
        payload.elements[3].exercise_number = 2;
        payload.elements[4].exercise_number = 3;
        payload.elements[5].exercise_number = 2;
        payload.elements[6].exercise_number = 3;
        payload.elements[7].exercise_number = 2;

        payload.elements[3].set_number = 1;
        payload.elements[4].set_number = 1;
        payload.elements[5].set_number = 2;
        payload.elements[6].set_number = 2;
        payload.elements[7].set_number = 3;
        payload.elements[3..=7].iter_mut().for_each(|item| {
            item.super_set = Some(1);
        });

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
        );

        let resp = create_workout_execution(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        if let Body::Text(body) = response.into_body() {
            let template: Result<WkExecutionWithElements, _> = serde_json::from_str(&body);
            assert!(template.is_ok()); 
            assert_eq!(template.unwrap().workout_execution.date, chrono::Utc::now().date_naive());
        }
    }

    #[tokio::test]
    async fn test_create_workout_execution_invalid_ids() {
        let (connector, _container) = pg_container().await;
        let (user_id, mut payload) = setup_execution(&connector, 1).await;
        let jwt = test_jwt(user_id);

        // 3 x exercise 1
        payload.elements[0].exercise_id = Uuid::new_v4();

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
        );

        let resp = create_workout_execution(req, &connector).await;
        assert!(resp.is_ok());
        let response = resp.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_workout_execution_invalid_set_number() {
        let (connector, _container) = pg_container().await; 
        {
            let (user_id, mut payload) = setup_execution(&connector, 3).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].set_number = 1;
            payload.elements[1].set_number = 3; // switched around
            payload.elements[2].set_number = 2;


            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}The set_number is strictly sequential for all exercise number values, when sorted by position{}", BASE_ERROR, DOC_LINK));
            }
        }     

        {
            let (user_id, mut payload) = setup_execution(&connector, 4).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = 1;
            payload.elements[1].exercise_number = 2;
            payload.elements[2].exercise_number = 2;
            payload.elements[3].exercise_number = 3;
            
            payload.elements[0].set_number = 1;
            payload.elements[1].set_number = 1;
            payload.elements[2].set_number = 2;
            payload.elements[3].set_number = 1;

            payload.elements[0..=3].iter_mut().for_each(|item| {
                item.super_set = Some(1);
            });

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}The set_number can't decrease for rows in a superset, when sorted by position{}", BASE_ERROR, DOC_LINK));
            }
        }
    }
    #[tokio::test]
    async fn test_create_workout_execution_invalid_exercise_number() {
        let (connector, _container) = pg_container().await; 
        {
            let (user_id, mut payload) = setup_execution(&connector, 3).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = 2;
            payload.elements[1].exercise_number = 3;
            payload.elements[2].exercise_number = 4;


            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}The element's exercise_number must be sequential, starting from 1 (repetitions allowed){}", BASE_ERROR, DOC_LINK));
            }
        }     

        {
            let (user_id, mut payload) = setup_execution(&connector, 3).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = -1;
            payload.elements[1].exercise_number = 3;
            payload.elements[2].exercise_number = 4;

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}All reps, set_number and time values must be at least 1. No values can't be negative{}", BASE_ERROR, DOC_LINK));
            }
        }   
    }
    #[tokio::test]
    async fn test_create_workout_execution_invalid_super_set() {
        let (connector, _container) = pg_container().await; 
        {
            let (user_id, mut payload) = setup_execution(&connector, 4).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = 1;
            payload.elements[1].exercise_number = 2;
            payload.elements[2].exercise_number = 3;
            payload.elements[3].exercise_number = 4;
            
            payload.elements[0].super_set = Some(1);
            payload.elements[1].super_set = Some(1);
            payload.elements[2].super_set = Some(3); // Should be 2
            payload.elements[3].super_set = Some(3);

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}The non-null super_set values must be sequential, starting from 1 with repetitions{}", BASE_ERROR, DOC_LINK));
            }
        }     

        {
            let (user_id, mut payload) = setup_execution(&connector, 2).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = 1;
            payload.elements[1].exercise_number = 1;
            payload.elements[1].set_number = 2;
            
            payload.elements[0].super_set = Some(1); // Cant be same exercise number
            payload.elements[1].super_set = Some(1);

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}There must be at least 2 different exercise numbers per super_set group{}", BASE_ERROR, DOC_LINK));
            }
        }    
        {
            let (user_id, mut payload) = setup_execution(&connector, 3).await;
            let jwt = test_jwt(user_id);

            payload.elements[0].exercise_number = 1;
            payload.elements[1].exercise_number = 2;
            payload.elements[2].exercise_number = 3;

            payload.elements[0].super_set = Some(1); 
            payload.elements[2].super_set = Some(1); // Cant have non consecutive rows in the same
                                                     // super set

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

            let resp = create_workout_execution(req, &connector).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            if let Body::Text(body) = response.into_body() {
                let body_string: String = body;
                let unescaped_body = serde_json::from_str::<String>(&body_string).unwrap();
                assert_eq!(unescaped_body, format!("{}In each super_set group all position values must be sequential{}", BASE_ERROR, DOC_LINK));
            }
        }
    }
}


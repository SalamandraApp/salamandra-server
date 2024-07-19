use lambda_http::{Error, Request, Response, Body, RequestExt};
use lambda_http::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

use salamandra_server::lib::db::exercises_db::validate_exercises;
use salamandra_server::lib::db::workout_templates_db::{insert_workout_template, delete_workout_template};
use salamandra_server::lib::db::wk_template_elements_db::insert_batch_wk_template_elements;
use salamandra_server::lib::db::DBPool;
use salamandra_server::lib::models::workout_templates_models::{NewWorkoutTemplate, WkTemplateWithElements, WorkoutTemplate};
use salamandra_server::lib::models::wk_template_elements_models::NewWkTemplateElement;
use salamandra_server::lib::utils::handlers::{build_resp, extract_sub};
use salamandra_server::lib::errors::DBError;


#[derive(Serialize, Deserialize)]
struct CreateWkTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub date_created: chrono::NaiveDate,
    pub elements: Vec<WkTemplateElementRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct WkTemplateElementRequest {
    exercise_id: Uuid,
    position: usize,
    reps: usize,
    sets: usize,
    weight: Option<f32>,
    rest: usize,
    super_set: Option<usize>,
}


pub async fn create_workout_template(event: Request, test_db: Option<DBPool>) -> Result<Response<Body>, Error> {

    let user_id = match event.path_parameters().first("user_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid user_id")),
    };

    match extract_sub(event.headers(), Some(user_id)) {
        Ok(_) => (),
        Err(resp) => return Ok(resp)
    };

    let body = match event.into_body() {
        Body::Text(body) => body,
        _ => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload")),
    };
    let req: CreateWkTemplateRequest = match serde_json::from_str(&body) {
        Ok(req) => req,
        Err(_) => return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload")),
    };

    // Check input
    // - Date
    // - Number of elements
    // - Position and super set
    let n = req.elements.len();
    if chrono::Utc::now().date_naive() < req.date_created
        || n == 0 
        || !valid_position_super_set(&req.elements) {
                return Ok(build_resp(StatusCode::BAD_REQUEST, "Invalid payload: see https://github.com/SalamandraApp/salamandra-server/wiki/Workout-templates-API#createwktemplaterequest for the correct format"));
    }

    let exercise_ids: HashSet<Uuid> = req.elements.iter().map(|element| element.exercise_id).collect();
    match validate_exercises(exercise_ids.into_iter().collect(), test_db.clone()).await {
        Ok(valid) => {
            if !valid {
                return Ok(build_resp(StatusCode::NOT_FOUND, "One or more exercise IDs do not reference existing exercises"));
            }
        }
        Err(_) => return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
    }
    let new_workout_template = NewWorkoutTemplate {
        user_id: user_id.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        date_created: req.date_created,
    };
    // Insert template
    let new_workout_template: WorkoutTemplate = match insert_workout_template(&new_workout_template, test_db.clone()).await {
        Ok(template) => template,
        Err(DBError::UniqueViolation(mes)) => return Ok(build_resp(StatusCode::CONFLICT, mes)),
        Err(_) => return Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
    };

    let new_workout_template_id = new_workout_template.id;
    // Create template elements
    let mut new_template_elements: Vec<NewWkTemplateElement> = Vec::new();
    for i in 0..n {
        let new_element = NewWkTemplateElement {
            workout_template_id: new_workout_template_id.clone(),
            exercise_id: req.elements[i].exercise_id,
            position: req.elements[i].position as i32,
            reps: req.elements[i].reps as i32,
            sets: req.elements[i].sets as i32,
            weight: req.elements[i].weight,
            rest: req.elements[i].rest as i32,
            super_set: req.elements[i].super_set.map(|s| s as i32),
        };
        new_template_elements.push(new_element);
    }

    // Insert template elements
    match insert_batch_wk_template_elements(&new_template_elements, test_db.clone()).await {
        Ok(elements) => {
            let response = WkTemplateWithElements {
                workout_template: new_workout_template,
                elements,
            };
            Ok(build_resp(StatusCode::CREATED, response))
        },
        Err(DBError::ConnectionError(_)) => Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, "")),
        Err(_) => {
            let _ = delete_workout_template(user_id, new_workout_template_id, test_db).await;
            Ok(build_resp(StatusCode::INTERNAL_SERVER_ERROR, ""))
        }
    }
}


fn valid_position_super_set(items: &[WkTemplateElementRequest]) -> bool {
    if items.is_empty() {
        return true;
    }
   
    // Sets and reps over 0
    if items.iter().any(|item| item.sets == 0 || item.reps == 0) {
        return false;
    }

    // Sequential position 
    let mut positions: Vec<usize> = items.iter().map(|item| item.position).collect();
    positions.sort_unstable();
    if positions != (1..items.len() + 1).collect::<Vec<_>>() { 
        return false;
    }

    // Group by superset
    let mut super_set_map: HashMap<Option<usize>, Vec<&WkTemplateElementRequest>> = HashMap::new();
    for item in items {
        super_set_map.entry(item.super_set).or_insert_with(Vec::new).push(item);
    }

    // Ensure super set values start at 0 and are sequential
    let mut super_set_values: Vec<usize> = super_set_map.keys().filter_map(|&k| k).collect();
    super_set_values.sort_unstable();
    if super_set_values != (1..super_set_values.len() + 1).collect::<Vec<_>>() {
        return false;
    }

    // Super set group
    for (super_set, group) in super_set_map {
        if let Some(_) = super_set {
            // At least 2 exercises
            if group.len() < 2 {
                return false;
            }

            // Sequential positions within superset
            let mut ss_positions: Vec<usize> = group.iter().map(|item| item.position).collect();
            ss_positions.sort_unstable();
            if ss_positions != (ss_positions[0]..ss_positions[0] + group.len()).collect::<Vec<_>>() {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;
    use lambda_http::http::header::{AUTHORIZATION, HeaderValue};
    use salamandra_server::lib::utils::tests::{insert_helper, pg_container, test_jwt, Items};

    async fn setup_template(db_pool: DBPool) -> (Uuid, CreateWkTemplateRequest) {
        let user_id = insert_helper(1, Items::Users, db_pool.clone(), None).await[0];
        let exercise_id = insert_helper(1, Items::Exercises, db_pool, None).await[0];

        let mut elements = Vec::new();
        for position in 1..5 {
            elements.push(WkTemplateElementRequest {
                exercise_id,
                position,
                reps: 1,
                sets: 1,
                weight: Some(0.0),
                super_set: None,
                rest: 0,
            });
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
        let (db_pool, _container) = pg_container().await;
        let (user_id, payload) = setup_template(db_pool.clone()).await;
        let jwt = test_jwt(user_id);

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

        let resp = create_workout_template(req, Some(db_pool)).await;
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
        let (db_pool, _container) = pg_container().await;
        let user_id = Uuid::new_v4();
        let jwt = test_jwt(user_id);

        let mut req = Request::default();
        req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
        let req = req.with_path_parameters(
            HashMap::from([("user_id".to_string(), user_id.to_string())])
            );

        let resp = create_workout_template(req, Some(db_pool)).await;
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

        let (db_pool, _container) = pg_container().await;
        { // ------ Wrong fields
            let (user_id, _payload) = setup_template(db_pool.clone()).await;
            let payload = BadPayload {test: 1};
            let jwt = test_jwt(user_id);

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
                );

            let resp = create_workout_template(req, Some(db_pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        { // ------ Invalid date
            let (user_id, mut payload) = setup_template(db_pool.clone()).await;
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

            let resp = create_workout_template(req, Some(db_pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        { // ------ Wrong exercise ids 
            let (user_id, mut payload) = setup_template(db_pool.clone()).await;
            let jwt = test_jwt(user_id);
            payload.elements[0].exercise_id = Uuid::new_v4();

            let mut req = Request::default();
            req.headers_mut().insert(AUTHORIZATION, HeaderValue::from_str(&jwt).unwrap());
            *req.body_mut() = Body::from(to_string(&payload).expect("Error"));
            let req = req.with_path_parameters(
                HashMap::from([("user_id".to_string(), user_id.to_string())])
                );

            let resp = create_workout_template(req, Some(db_pool.clone())).await;
            assert!(resp.is_ok());
            let response = resp.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND)
        }

        { // ------ Invalid element positions 
            let (user_id1, mut payload1) = setup_template(db_pool.clone()).await;
            let (user_id2, mut payload2) = setup_template(db_pool.clone()).await;
            let (user_id3, mut payload3) = setup_template(db_pool.clone()).await;

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

                let resp = create_workout_template(req, Some(db_pool.clone())).await;
                assert!(resp.is_ok());
                let response = resp.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST)
            }
        }

        { // ------ Invalid element superset 
            let (user_id1, mut payload1) = setup_template(db_pool.clone()).await;
            let (user_id2, mut payload2) = setup_template(db_pool.clone()).await;
            let (user_id3, mut payload3) = setup_template(db_pool.clone()).await;

            // Not sequential super set id
            payload1.elements[0].super_set = Some(1);
            payload1.elements[1].super_set = Some(1);
            payload1.elements[2].super_set = Some(3);
            payload1.elements[3].super_set = Some(3);

            // Only 1 exercise in superset
            payload2.elements[0].super_set = Some(0);

            // Not sequential in superset
            payload3.elements[0].super_set = Some(0);
            payload3.elements[2].super_set = Some(0);

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
                *req.body_mut() = Body::from(to_string(&payload).expect("Error serializing payload"));
                let req = req.with_path_parameters(
                    HashMap::from([("user_id".to_string(), user_id.to_string())])
                    );

                let resp = create_workout_template(req, Some(db_pool.clone())).await;
                assert!(resp.is_ok());
                let response = resp.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST)

            }
        }
    }
}

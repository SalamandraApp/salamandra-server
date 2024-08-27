use testcontainers::ContainerAsync;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use testcontainers_modules::{
    postgres, 
    testcontainers::runners::AsyncRunner
};
use crate::lib::db::wk_execution_elements_db::insert_batch_wk_execution_elements;
use crate::lib::db::workout_executions_db::insert_workout_execution;
use crate::lib::models::exercise_models::NewExercise;
use crate::lib::models::user_models::User;
use crate::lib::models::wk_execution_elements_models::NewWkExecutionElement;
use crate::lib::models::workout_execution_models::NewWorkoutExecution;
use crate::lib::models::workout_templates_models::NewWorkoutTemplate;
use crate::lib::models::wk_template_elements_models::NewWkTemplateElement;
use crate::lib::db::DBConnector;
use crate::lib::db::exercises_db::insert_exercise;
use crate::lib::db::users_db::insert_user;
use crate::lib::db::workout_templates_db::insert_workout_template;
use crate::lib::db::wk_template_elements_db::insert_batch_wk_template_elements;
 
pub const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations = diesel_async_migrations::embed_migrations!();


pub async fn pg_container() -> (DBConnector, ContainerAsync<postgres::Postgres>) {
    let db_name = random_string();
    let container = postgres::Postgres::default()
        .with_db_name(&db_name).start().await.unwrap();
    let endpoint = format!(
        "postgres://postgres:postgres@{}:{}/{}",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(5432).await.unwrap(),
        db_name
    );
    let connector = DBConnector{test_endpoint: Some(endpoint)};
    let mut conn = connector.rds_connection().await.expect("Error connecting to test container");
    MIGRATIONS.run_pending_migrations(&mut conn).await.expect("Error running migrations");
    return (connector, container);
}


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn test_jwt(user_id: Uuid) -> String {
    let my_claims = Claims {
        sub: user_id.to_string(),
        exp: 10000000000, // Expiration timestamp
    };

    let encoding_key = EncodingKey::from_secret("secret".as_ref());
    let token = encode(&Header::default(), &my_claims, &encoding_key).unwrap();
    format!("Bearer {}", token)
}

pub async fn insert_helper(n: usize, items: Items, connector: &DBConnector, name_prefix: Option<String>) -> Vec<Uuid> {
    let mut ids = Vec::new();
    match items {
        Items::Users => {
            for _ in 0..n {
                let random = random_string();
                let name = match &name_prefix {
                    Some(prefix) => format!("{}_{}", prefix, random),
                    None => random,
                };

                let new_user = User {
                    username: name,
                    ..Default::default()
                };
                let insert_res = insert_user(&new_user, connector).await;
                ids.push(insert_res.unwrap().id);
            }
        },
        Items::Exercises => {
            for _ in 0..n {
                let random = random_string();
                let name = match &name_prefix {
                    Some(prefix) => format!("{}_{}", prefix, random),
                    None => random,
                };

                let new_user = NewExercise{
                    name,
                    ..Default::default()
                };
                let insert_res = insert_exercise(&new_user, connector).await;
                ids.push(insert_res.unwrap().id);
            }
        },
        Items::WkTemplates => {
            let new_user_id = Box::pin(insert_helper(1, Items::Users, connector, None)).await[0];
            for _ in 0..n {
                let new_template = NewWorkoutTemplate { user_id: new_user_id.clone(), ..Default::default() };
                let insert_res = insert_workout_template(&new_template, connector).await;
                ids.push(insert_res.unwrap().id);
            }
        },
        Items::WkExecutions => {
            let new_template_id = Box::pin(insert_helper(1, Items::WkTemplates, connector, None)).await[0];
            for _ in 0..n {
                let new_execution = NewWorkoutExecution { workout_template_id: new_template_id.clone(), ..Default::default() };
                let insert_res = insert_workout_execution(&new_execution, connector).await;
                ids.push(insert_res.unwrap().id);
            }
        },

        Items::WkTemplateElements => {
            let workout_template_id = Box::pin(insert_helper(1, Items::WkTemplates, connector, None)).await.into_iter().next().unwrap();
            let exercise_ids = Box::pin(insert_helper(n, Items::Exercises, connector, Some("Push-up".to_string()))).await;
            for i in 0..n {
                let new_element = NewWkTemplateElement{ workout_template_id, exercise_id: exercise_ids[i], ..Default::default() };
                let insert_res = insert_batch_wk_template_elements(&vec![new_element], connector).await;
                ids.push(insert_res.unwrap().into_iter().next().unwrap().id);
            }
        },
        Items::WkExecutionElements => {
            let workout_execution_id = Box::pin(insert_helper(1, Items::WkExecutions, connector, None)).await.into_iter().next().unwrap();
            let exercise_ids = Box::pin(insert_helper(n, Items::Exercises, connector, Some("Push-up".to_string()))).await;
            for i in 0..n {
                let new_element = NewWkExecutionElement{ workout_execution_id, exercise_id: exercise_ids[i], ..Default::default() };
                let insert_res = insert_batch_wk_execution_elements(&vec![new_element], connector).await;
                ids.push(insert_res.unwrap().into_iter().next().unwrap().id);
            }
        },
    }
    ids
}

pub enum Items {
    Users,
    Exercises,
    WkTemplates,
    WkTemplateElements,
    WkExecutions,
    WkExecutionElements,
}


fn random_string() -> String {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    format!("{}", timestamp)
}

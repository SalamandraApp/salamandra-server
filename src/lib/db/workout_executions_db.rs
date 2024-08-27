use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::workoutexecutions::dsl::*;
use crate::lib::models::workout_execution_models::{NewWorkoutExecution, WorkoutExecution};
use crate::lib::errors::DBError;

use super::DBConnector;

/// Insert single workout execution
pub async fn insert_workout_execution(new_execution: &NewWorkoutExecution, connector: &DBConnector) -> Result<WorkoutExecution, DBError> {
    let mut conn = connector.rds_connection().await?;

    diesel::insert_into(workoutexecutions)
        .values(new_execution)
        .returning(WorkoutExecution::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Workout execution already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })
}

/// Fetch single exercution from database
pub async fn lookup_workout_execution(execution_id: Uuid, connector: &DBConnector) -> Result<WorkoutExecution, DBError> {
    let mut conn = connector.rds_connection().await?;
    let workout_execution = workoutexecutions.find(execution_id)
        .first::<WorkoutExecution>(&mut conn)
        .await
        .map_err(|error| {
            if error == Error::NotFound {
                DBError::ItemNotFound("No execution exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(workout_execution)
}

use crate::schema::workouttemplates::dsl::{
    workouttemplates,
    id as workout_template_id_,
    user_id as user_id_,
};

/// Lookup workout execution that corresponds to the current user
pub async fn lookup_workout_execution_validated(execution_id: Uuid, user_id: Uuid, connector: &DBConnector) -> Result<WorkoutExecution, DBError> {
    let mut conn = connector.rds_connection().await?;
    let res = workoutexecutions
        .filter(id.eq(execution_id))
        .inner_join(workouttemplates.on(workout_template_id.eq(workout_template_id_)))
        .filter(user_id_.eq(user_id))
        .select(workoutexecutions::all_columns())
        .first::<WorkoutExecution>(&mut conn)
        .await
        .map_err(|error| match error {
            Error::NotFound => {
                DBError::ItemNotFound("No execution exists with the corresponding id".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })?;
    Ok(res)
}

/// Delete workout execution by id
pub async fn delete_workout_execution(execution_id: uuid::Uuid, connector: &DBConnector) -> Result<usize, DBError> {
    let mut conn = connector.rds_connection().await?;

    diesel::delete(workoutexecutions.filter(id.eq(execution_id)))
        .execute(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Insert wrong id
    // * Insert and look up
    // * Lookup non existing
    // * Delete success, non existing

    #[tokio::test]
    async fn test_insert_workout_execution_wrong_template_id() {
        let (connector, _container) = pg_container().await;

        // Create a new workout_execution object to insert
        let new_workout_execution = NewWorkoutExecution{..Default::default()};

        let insert_res = insert_workout_execution(&new_workout_execution, &connector).await;
        assert!(insert_res.is_err());
    }

    #[tokio::test]
    async fn test_insert_lookup_workout_execution() {
        let (connector, _container) = pg_container().await;

        let new_template_id = insert_helper(1, Items::WkTemplates, &connector, None).await[0];
        let new_workout_execution = NewWorkoutExecution {workout_template_id: new_template_id, ..Default::default()};

        let insert_res = insert_workout_execution(&new_workout_execution, &connector).await;
        assert!(insert_res.is_ok());
        let inserted_workout_execution = insert_res.unwrap();

        let read_res = lookup_workout_execution(inserted_workout_execution.id, &connector).await;
        assert!(read_res.is_ok());
    }

    #[tokio::test]
    async fn test_lookup_workout_execution_non_existing() {
        let (connector, _container) = pg_container().await;

        // Look up non existing workout_execution
        let read_res = lookup_workout_execution(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_err());
    }
    
    #[tokio::test]
    async fn test_delete_workout_execution_success() {
        let (connector, _container) = pg_container().await;

        let new_execution_id = insert_helper(1, Items::WkExecutions, &connector, None).await[0];

        let read_res1 = lookup_workout_execution(new_execution_id.clone(), &connector).await;
        assert!(read_res1.is_ok());

        let delete_res = delete_workout_execution(new_execution_id.clone(), &connector).await;
        assert!(delete_res.is_ok());
        let read_res2 = lookup_workout_execution(new_execution_id, &connector).await;
        assert!(read_res2.is_err());

    }

    #[tokio::test]
    async fn test_delete_workout_execution_non_exisiting() {
        let (connector, _container) = pg_container().await;

        let delete_res = delete_workout_execution(Uuid::new_v4(), &connector).await;
        assert!(delete_res.is_ok());
        assert_eq!(delete_res.unwrap(), 0);
    }
}

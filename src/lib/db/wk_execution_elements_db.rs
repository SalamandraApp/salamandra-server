use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::wkexecutionelements::dsl::*;
use crate::lib::models::wk_execution_elements_models::{NewWkExecutionElement, WkExecutionElement, WkExecutionElementFull};
use crate::lib::errors::DBError;

use super::DBConnector;

/// Inserts a batch of new workflow execution elements into the database.
pub async fn insert_batch_wk_execution_elements(new_elements: &Vec<NewWkExecutionElement>, connector: &DBConnector) -> Result<Vec<WkExecutionElement>, DBError> {

    let mut conn = connector.rds_connection().await?;
    diesel::insert_into(wkexecutionelements)
        .values(new_elements)
        .get_results(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Execution element already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })
}

/// Returns a user with the corresponding ID, or an error if not found.
pub async fn lookup_wk_execution_element(wk_execution_element_id: Uuid, connector: &DBConnector) -> Result<WkExecutionElement, DBError> {

    let mut conn = connector.rds_connection().await?;
    let wk_execution_element = wkexecutionelements.find(wk_execution_element_id)
        .first::<WkExecutionElement>(&mut conn)
        .await
        .map_err(|error| {
            if error == Error::NotFound {
                DBError::ItemNotFound("No wk_execution_element exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(wk_execution_element)
}

use crate::schema::exercises::dsl::{
    exercises, 
    id as exercise_id_,
    name as exercise_name,
    main_muscle_group,
    secondary_muscle_group,
    necessary_equipment,
    exercise_type,
};

// Return all execution elements with given execution id including exercise info
pub async fn select_wk_execution_element_by_execution_full(wk_execution_id: Uuid, connector: &DBConnector) -> Result<Vec<WkExecutionElementFull>, DBError> {

    let mut conn = connector.rds_connection().await?;

    wkexecutionelements
        .filter(workout_execution_id.eq(wk_execution_id))
        .inner_join(exercises.on(exercise_id.eq(exercise_id_)))
        .select((
            id,
            workout_execution_id,
            position,
            exercise_number,
            reps,
            set_number,
            weight,
            rest,
            super_set,
            time,
            exercise_id_,
            exercise_name,
            main_muscle_group,
            secondary_muscle_group,
            necessary_equipment,
            exercise_type
        ))
        .load::<WkExecutionElementFull>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


// Return all execution elements with given execution id
pub async fn select_wk_execution_element_by_execution(wk_execution_id: Uuid, connector: &DBConnector) -> Result<Vec<WkExecutionElement>, DBError> {

    let mut conn = connector.rds_connection().await?;

    wkexecutionelements
        .filter(workout_execution_id.eq(wk_execution_id))
        .select((
            id,
            workout_execution_id,
            exercise_id,
            position,
            exercise_number,
            reps,
            set_number,
            weight,
            rest,
            super_set,
            time,
        ))
        .load::<WkExecutionElement>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Lookup non existing
    // * Insert and lookup
    // * Insert wrong execution id
    // * Select multiple and none

    #[tokio::test]
    async fn test_lookup_execution_element_non_existing() {
        let (connector, _container) = pg_container().await;

        let read_res = lookup_wk_execution_element(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_insert_batch_lookup_execution_element() {
        let (connector, _container) = pg_container().await;

        let new_exercise_id = insert_helper(1, Items::Exercises, &connector, None).await[0];
        let new_execution_id = insert_helper(1, Items::WkExecutions, &connector, None).await[0];        
        let mut new_elements : Vec<NewWkExecutionElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkExecutionElement {
                workout_execution_id: new_execution_id.clone(),
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_execution_elements(&new_elements, &connector).await;
        assert!(insert_res.is_ok());
        let inserted_element = &insert_res.unwrap()[0];
    
        let read_res = lookup_wk_execution_element(inserted_element.id, &connector).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_batch_execution_element_wrong_workout_execution_id() {
        let (connector, _container) = pg_container().await;

        // Workout execution id references nothing
        let new_exercise_id = insert_helper(1, Items::Exercises, &connector, None).await[0];
        let mut new_elements : Vec<NewWkExecutionElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkExecutionElement {
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_execution_elements(&new_elements, &connector).await;
        assert!(matches!(insert_res, Err(DBError::OperationError(_))));
    }


    #[tokio::test]
    async fn test_select_wk_execution_element_by_execution_full_none() {
        let (connector, _container) = pg_container().await;

        let read_res = select_wk_execution_element_by_execution_full(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_ok());
        assert_eq!(read_res.unwrap().len(), 0);
    }
    
    #[tokio::test]
    async fn test_select_wk_execution_element_by_execution_full_multiple() {
        let (connector, _container) = pg_container().await;
        let insert_res = insert_helper(4, Items::WkExecutionElements, &connector, None).await;
        let new_workout_execution_id = lookup_wk_execution_element(insert_res[0], &connector).await.unwrap().workout_execution_id;

        let read_res = select_wk_execution_element_by_execution_full(new_workout_execution_id, &connector).await;
        assert!(read_res.is_ok());
        let vector = read_res.clone().unwrap();
        assert_eq!(vector.len(), 4);
    }
}

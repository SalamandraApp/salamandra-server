use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::wktemplateelements::dsl::*;
use crate::lib::models::wk_template_elements_models::{NewWkTemplateElement, WkTemplateElement, WkTemplateElementFull};
use crate::lib::errors::DBError;

use super::DBConnector;


/// Returns a wk_template_element with the corresponding ID, or an error if not found.
pub async fn insert_batch_wk_template_elements(new_elements: &Vec<NewWkTemplateElement>, connector: &DBConnector) -> Result<Vec<WkTemplateElement>, DBError> {

    let mut conn = connector.rds_connection().await?;

    diesel::insert_into(wktemplateelements)
        .values(new_elements)
        .get_results(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Template element already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })

}


/// Returns a wk_template_element with the corresponding ID, or an error if not found.
pub async fn lookup_wk_template_element(wk_template_element_id: Uuid, connector: &DBConnector) -> Result<WkTemplateElement, DBError> {

    let mut conn = connector.rds_connection().await?;
    let wk_template_element = wktemplateelements.find(wk_template_element_id)
        .first::<WkTemplateElement>(&mut conn)
        .await
        .map_err(|error| {
            if error == Error::NotFound {
                DBError::ItemNotFound("No wk_template_element exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(wk_template_element)
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

/// Selects detailed template elements by workout template ID.
pub async fn select_wk_template_element_by_template_full(wk_template_id: Uuid, connector: &DBConnector) -> Result<Vec<WkTemplateElementFull>, DBError> {

    let mut conn = connector.rds_connection().await?;

    wktemplateelements
        .filter(workout_template_id.eq(wk_template_id))
        .inner_join(exercises.on(exercise_id.eq(exercise_id_)))
        .select((
            id,
            workout_template_id,
            position,
            reps,
            sets,
            weight,
            rest,
            super_set,
            exercise_id_,
            exercise_name,
            main_muscle_group,
            secondary_muscle_group,
            necessary_equipment,
            exercise_type
        ))
        .load::<WkTemplateElementFull>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}

/// Selects detailed template elements by workout template ID.
///
/// This function retrieves detailed template elements for a given workout template ID
pub async fn select_wk_template_element_by_template(wk_template_id: Uuid, connector: &DBConnector) -> Result<Vec<WkTemplateElement>, DBError> {

    let mut conn = connector.rds_connection().await?;

    wktemplateelements
        .filter(workout_template_id.eq(wk_template_id))
        .select((
            id,
            workout_template_id,
            exercise_id,
            position,
            reps,
            sets,
            weight,
            rest,
            super_set,
        ))
        .load::<WkTemplateElement>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Insert batch and lookup
    // * Insert wrong id
    // * Lookup non existing
    // * Select full and none
        
    #[tokio::test]
    async fn test_insert_batch_lookup_template_element() {
        let (connector, _container) = pg_container().await;

        let new_exercise_id = insert_helper(1, Items::Exercises, &connector, None).await[0];
        let new_template_id = insert_helper(1, Items::WkTemplates, &connector, None).await[0];        
        let mut new_elements : Vec<NewWkTemplateElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkTemplateElement {
                workout_template_id: new_template_id.clone(),
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_template_elements(&new_elements, &connector).await;
        assert!(insert_res.is_ok());
        let inserted_element = &insert_res.unwrap()[0];
    
        let read_res = lookup_wk_template_element(inserted_element.id, &connector).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_batch_template_element_wrong_workout_template_id() {
        let (connector, _container) = pg_container().await;

        // Workout template id references nothing
        let new_exercise_id = insert_helper(1, Items::Exercises, &connector, None).await[0];
        let mut new_elements : Vec<NewWkTemplateElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkTemplateElement {
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_template_elements(&new_elements, &connector).await;
        assert!(matches!(insert_res, Err(DBError::OperationError(_))));
    }

    #[tokio::test]
    async fn test_lookup_template_element_non_existing() {
        let (connector, _container) = pg_container().await;

        let read_res = lookup_wk_template_element(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_select_wk_template_element_by_template_full_none() {
        let (connector, _container) = pg_container().await;

        let read_res = select_wk_template_element_by_template_full(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_ok());
        assert_eq!(read_res.unwrap().len(), 0);
    }
    
    #[tokio::test]
    async fn test_select_wk_template_element_by_template_full_multiple() {
        let (connector, _container) = pg_container().await;
        let insert_res = insert_helper(4, Items::WkTemplateElements, &connector, None).await;
        let new_workout_template_id = lookup_wk_template_element(insert_res[0], &connector).await.unwrap().workout_template_id;

        let read_res = select_wk_template_element_by_template_full(new_workout_template_id, &connector).await;
        assert!(read_res.is_ok());
        let vector = read_res.clone().unwrap();
        assert_eq!(vector.len(), 4);
    }
}

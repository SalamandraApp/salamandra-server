use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::wktemplateelements::dsl::*;
use crate::lib::models::wk_template_elements_models::{NewWkTemplateElement, WkTemplateElement, WkTemplateElementDetailed};
use crate::lib::errors::DBError;

use super::{get_db_pool, DBPool};


pub async fn insert_batch_wk_template_elements(new_elements: &Vec<NewWkTemplateElement>, test_db: Option<DBPool>) -> Result<Vec<WkTemplateElement>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

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
///
/// This function performs a lookup for a wk_template_element by its primary key (UUID).
/// If the wk_template_element is found, it is returned. Otherwise, an appropriate error
/// is returned.
pub async fn lookup_wk_template_element(wk_template_element_id: Uuid, test_db: Option<DBPool>) -> Result<WkTemplateElement, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
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
///
/// This function retrieves detailed template elements for a given workout template ID
/// by performing an inner join between the `templateelements` and `exercises` tables.
/// It returns a vector of `TemplateElementDetailed` structs containing detailed information
/// about each template element.
pub async fn select_wk_template_element_detailed_by_template(wk_template_id: Uuid, test_db: Option<DBPool>) -> Result<Vec<WkTemplateElementDetailed>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

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
        .load::<WkTemplateElementDetailed>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    #[tokio::test]
    async fn test_insert_batch_lookup_template_element() {
        let (db_pool, _container) = pg_container().await;

        let new_exercise_id = insert_helper(1, Items::Exercises, db_pool.clone(), None).await[0];
        let new_template_id = insert_helper(1, Items::WkTemplates, db_pool.clone(), None).await[0];        
        let mut new_elements : Vec<NewWkTemplateElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkTemplateElement {
                workout_template_id: new_template_id.clone(),
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_template_elements(&new_elements, Some(db_pool.clone())).await;
        assert!(insert_res.is_ok());
        let inserted_element = &insert_res.unwrap()[0];
    
        let read_res = lookup_wk_template_element(inserted_element.id, Some(db_pool)).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_batch_template_element_wrong_workout_template_id() {
        let (db_pool, _container) = pg_container().await;

        // Workout template id references nothing
        let new_exercise_id = insert_helper(1, Items::Exercises, db_pool.clone(), None).await[0];
        let mut new_elements : Vec<NewWkTemplateElement> = Vec::new();
        for _ in 0..6 {
            let new_element = NewWkTemplateElement {
                exercise_id: new_exercise_id.clone(),
                ..Default::default()
            };
            new_elements.push(new_element);
        }

        let insert_res = insert_batch_wk_template_elements(&new_elements, Some(db_pool)).await;
        assert!(matches!(insert_res, Err(DBError::OperationError(_))));
    }

    #[tokio::test]
    async fn test_lookup_template_element_non_existing() {
        let (db_pool, _container) = pg_container().await;

        let read_res = lookup_wk_template_element(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_select_wk_template_element_detailed_by_template_none() {
        let (db_pool, _container) = pg_container().await;

        let read_res = select_wk_template_element_detailed_by_template(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_ok());
        assert_eq!(read_res.unwrap().len(), 0);
    }
    
    #[tokio::test]
    async fn test_select_wk_template_element_detailed_by_template_multiple() {
        let (db_pool, _container) = pg_container().await;
        let insert_res = insert_helper(4, Items::WkTemplateElements, db_pool.clone(), None).await;
        let new_workout_template_id = lookup_wk_template_element(insert_res[0], Some(db_pool.clone())).await.unwrap().workout_template_id;

        let read_res = select_wk_template_element_detailed_by_template(new_workout_template_id, Some(db_pool)).await;
        assert!(read_res.is_ok());
        let vector = read_res.clone().unwrap();
        assert_eq!(vector.len(), 4);
    }
}

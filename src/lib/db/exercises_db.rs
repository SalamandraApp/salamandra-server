use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::exercises::dsl::*;
use crate::lib::models::exercise_models::{Exercise, NewExercise};
use crate::lib::errors::DBError;

use super::DBConnector;

/// Inserts a new exercise into the database and returns the inserted user.
///
/// This function inserts a new exercise into the `exercises` table.
/// If the insertion is successful, the inserted `Exercise` is returned.
pub async fn insert_exercise(new_exercise: &NewExercise, connector: &DBConnector) -> Result<Exercise, DBError> {

    let mut conn = connector.rds_connection().await?;
    diesel::insert_into(exercises)
        .values(new_exercise)
        .returning(Exercise::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Exercise already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })
}

/// Returns a exercise with the corresponding ID, or an error if not found.
///
/// This function performs a lookup for a exercise by its primary key (UUID).
/// If the exercise is found, it is returned. Otherwise, an appropriate error
/// is returned.
pub async fn lookup_exercise(exercise_id: Uuid, connector: &DBConnector) -> Result<Exercise, DBError> {

    let mut conn = connector.rds_connection().await?;
    let exercise = exercises.find(exercise_id)
        .first::<Exercise>(&mut conn)
        .await
        .map_err(|error| {
            if error == Error::NotFound {
                DBError::ItemNotFound("No exercise exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(exercise)
}


/// Searches for exercises with names starting with the given term.
///
/// Case-insensitive search in the `exercises` table,
/// returning all exercises whose names begin with the specified term
pub async fn search_exercises(term: &str, connector: &DBConnector) -> Result<Vec<Exercise>, DBError> {

    let mut conn = connector.rds_connection().await?;
    let pattern = format!("{}%", term);
    exercises.filter(name.like(pattern))
        .load::<Exercise>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


/// Checks if all provided UUIDs are valid references to existing exercises.
///
/// This function verifies whether all given UUIDs correspond to existing exercises
/// in the database. It returns `true` if all UUIDs are valid, and `false` otherwise.
pub async fn validate_exercises(exercise_ids: Vec<Uuid>, connector: &DBConnector) -> Result<bool, DBError> {
    
    let mut conn = connector.rds_connection().await?;
    let n = exercise_ids.len();
    let found_uuids = exercises.filter(id.eq_any(&exercise_ids))
            .select(id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(|error| DBError::OperationError(error.to_string()))?;

    Ok(found_uuids.len() == n)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Insert and lookup inserted exercise
    // * Insert with duplicate PK
    // * Lookup non exisiting
    // * Search multiple and empty
    // * Validate all valid, no valid, some valid

    #[tokio::test]
    async fn test_insert_lookup_exercise() {
        let (connector, _container) = pg_container().await;

        // Create a new exercise object to insert
        let new_exercise = NewExercise {
            ..Default::default()
        };

        let insert_res = insert_exercise(&new_exercise, &connector).await;
        assert!(insert_res.is_ok());

        let new_exercise_id = insert_res.unwrap().id;

        let read_res = lookup_exercise(new_exercise_id, &connector).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_exercise_duplicate_exercisename() {
        let (connector, _container) = pg_container().await;

        // Create a new exercise object to insert
        let new_exercise1 = NewExercise {..Default::default()};
        let new_exercise2 = NewExercise {..Default::default()};

        let insert_res1 = insert_exercise(&new_exercise1, &connector).await;
        assert!(insert_res1.is_ok());
        let new_exercise_id = insert_res1.unwrap().id;

        let read_res = lookup_exercise(new_exercise_id, &connector).await;
        assert!(matches!(read_res, Ok(_new_exercise1)));

        let insert_res2 = insert_exercise(&new_exercise2, &connector).await;
        assert!(insert_res2.is_err());
    }
    #[tokio::test]
    async fn test_lookup_exercise_non_existing() {
        let (connector, _container) = pg_container().await;

        // Look up non existing exercise
        let read_res = lookup_exercise(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_search_exercises_multiple() {
        let (connector, _container) = pg_container().await;

        let exercise_ids = insert_helper(5, Items::Exercises, &connector, Some("TEST".into())).await;

        let pattern = "TEST";
        let search_res = search_exercises(pattern, &connector).await;
        assert!(search_res.is_ok());

        let exercise_vec = search_res.unwrap();
        let id_vec: Vec<Uuid> = exercise_vec.iter().map(|ex| ex.id.clone()).collect();
        assert_eq!(id_vec.len(), 5);
        assert_eq!(id_vec, exercise_ids);
    }

    #[tokio::test]
    async fn test_search_exercises_success_none() {
        let (connector, _container) = pg_container().await;

        let pattern = "Testing";
        let search_res = search_exercises(pattern, &connector).await;
        assert!(search_res.is_ok());

        let vec = search_res.unwrap();
        assert_eq!(vec.len(), 0, "Should have been 0 exercises");
    }

    #[tokio::test]
    async fn test_validate_exercises_none() {
        let (connector, _container) = pg_container().await;

        let uuids: Vec<Uuid> = Vec::new();

        let validate_res = validate_exercises(uuids, &connector).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), true);
    }

    #[tokio::test]
    async fn test_validate_exercises_all_valid() {
        let (connector, _container) = pg_container().await;

        let exercise_ids = insert_helper(5, Items::Exercises, &connector, Some("TEST".into())).await;

        let validate_res = validate_exercises(exercise_ids, &connector).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), true);
    }

    #[tokio::test]
    async fn test_validate_exercises_some_invalid() {
        let (connector, _container) = pg_container().await;

        let mut exercise_ids = insert_helper(5, Items::Exercises, &connector, Some("TEST".into())).await;
        exercise_ids.push(Uuid::new_v4());

        let validate_res = validate_exercises(exercise_ids, &connector).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), false);
    }
}

use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::exercises::dsl::*;
use crate::lib::models::exercise_models::{Exercise, NewExercise};
use crate::lib::errors::DBError;

use super::{get_db_pool, DBPool};

/// Inserts a new exercise into the database and returns the inserted user.
///
/// This function inserts a new exercise into the `exercises` table.
/// If the insertion is successful, the inserted `Exercise` is returned.
pub async fn insert_exercise(new_exercise: &NewExercise, test_db: Option<DBPool>) -> Result<Exercise, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

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
pub async fn lookup_exercise(exercise_id: Uuid, test_db: Option<DBPool>) -> Result<Exercise, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
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
/// This function performs a case-insensitive search in the `exercises` table,
/// returning all exercises whose names begin with the specified t
pub async fn search_exercises(term: &str, test_db: Option<DBPool>) -> Result<Vec<Exercise>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

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
pub async fn validate_exercises(exercise_ids: Vec<Uuid>, test_db: Option<DBPool>) -> Result<bool, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let n = exercise_ids.len();

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
    let res = exercises.filter(id.eq_any(&exercise_ids))
            .select(id)
            .load::<Uuid>(&mut conn)
            .await
            .map_err(|error| DBError::OperationError(error.to_string()));

    let found_uuids = res?;
    Ok(found_uuids.len() == n)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    #[tokio::test]
    async fn test_insert_lookup_exercise() {
        let (db_pool, _container) = pg_container().await;

        // Create a new exercise object to insert
        let new_exercise = NewExercise {
            ..Default::default()
        };

        let insert_res = insert_exercise(&new_exercise, Some(db_pool.clone())).await;
        assert!(insert_res.is_ok());

        let new_exercise_id = insert_res.unwrap().id;

        let read_res = lookup_exercise(new_exercise_id, Some(db_pool.clone())).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_exercise_duplicate_exercisename() {
        let (db_pool, _container) = pg_container().await;

        // Create a new exercise object to insert
        let new_exercise1 = NewExercise {..Default::default()};
        let new_exercise2 = NewExercise {..Default::default()};

        let insert_res1 = insert_exercise(&new_exercise1, Some(db_pool.clone())).await;
        assert!(insert_res1.is_ok());
        let new_exercise_id = insert_res1.unwrap().id;

        let read_res = lookup_exercise(new_exercise_id, Some(db_pool.clone())).await;
        assert!(matches!(read_res, Ok(_new_exercise1)));

        let insert_res2 = insert_exercise(&new_exercise2, Some(db_pool)).await;
        assert!(insert_res2.is_err());
    }
    #[tokio::test]
    async fn test_lookup_exercise_non_existing() {
        let (db_pool, _container) = pg_container().await;

        // Look up non existing exercise
        let read_res = lookup_exercise(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_search_exercises_multiple() {
        let (db_pool, _container) = pg_container().await;

        let exercise_ids = insert_helper(5, Items::Exercises, db_pool.clone(), Some("TEST".into())).await;

        let pattern = "TEST";
        let search_res = search_exercises(pattern, Some(db_pool)).await;
        assert!(search_res.is_ok());

        let exercise_vec = search_res.unwrap();
        let id_vec: Vec<Uuid> = exercise_vec.iter().map(|ex| ex.id.clone()).collect();
        assert_eq!(id_vec.len(), 5);
        assert_eq!(id_vec, exercise_ids);
    }

    #[tokio::test]
    async fn test_search_exercises_success_none() {
        let (db_pool, _container) = pg_container().await;

        let pattern = "Testing";
        let search_res = search_exercises(pattern, Some(db_pool)).await;
        assert!(search_res.is_ok());

        let vec = search_res.unwrap();
        assert_eq!(vec.len(), 0, "Should have been 0 exercises");
    }

    #[tokio::test]
    async fn test_validate_exercises_none() {
        let (db_pool, _container) = pg_container().await;

        let uuids: Vec<Uuid> = Vec::new();

        let validate_res = validate_exercises(uuids, Some(db_pool)).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), true);
    }

    #[tokio::test]
    async fn test_validate_exercises_all_valid() {
        let (db_pool, _container) = pg_container().await;

        let exercise_ids = insert_helper(5, Items::Exercises, db_pool.clone(), Some("TEST".into())).await;

        let validate_res = validate_exercises(exercise_ids, Some(db_pool)).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), true);
    }

    #[tokio::test]
    async fn test_validate_exercises_some_invalid() {
        let (db_pool, _container) = pg_container().await;

        let mut exercise_ids = insert_helper(5, Items::Exercises, db_pool.clone(), Some("TEST".into())).await;
        exercise_ids.push(Uuid::new_v4());

        let validate_res = validate_exercises(exercise_ids, Some(db_pool)).await;
        assert!(validate_res.is_ok());
        assert_eq!(validate_res.unwrap(), false);
    }
}

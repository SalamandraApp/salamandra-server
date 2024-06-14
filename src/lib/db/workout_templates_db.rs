use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::workouttemplates::dsl::*;
use crate::lib::models::workout_templates_models::{NewWorkoutTemplate, WorkoutTemplate};
use crate::lib::errors::DBError;

use super::{get_db_pool, DBPool};

/// Inserts a new workout_template into the database and returns the inserted workout_template.
///
/// This function inserts a new workout_template into the `workout_templates` table.
/// If the insertion is successful, the inserted `NewWorkoutTemplate` is returned.
pub async fn insert_workout_template(new_template: &NewWorkoutTemplate, test_db: Option<DBPool>) -> Result<WorkoutTemplate, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

    diesel::insert_into(workouttemplates)
        .values(new_template)
        .returning(WorkoutTemplate::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Workout template already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })
}


/// Returns a workout_template with the corresponding ID, or an error if not found.
///
/// This function performs a lookup for a workout_template by its primary key (UUID).
/// If the workout_template is found, it is returned. Otherwise, an appropriate error
/// is returned.
pub async fn lookup_workout_template(template_id: Uuid, test_db: Option<DBPool>) -> Result<WorkoutTemplate, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
    let workout_template = workouttemplates.find(template_id)
        .first::<WorkoutTemplate>(&mut conn)
        .await
        .map_err(|error| {
            if error == Error::NotFound {
                DBError::ItemNotFound("No workout_template exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(workout_template)
}

/// Selects workout template by user id
///
/// This function retrieves template for a given user id ID
/// It returns a vector of `WorkoutTemplate` structs
pub async fn select_workout_template_by_user(user_uuid: Uuid, test_db: Option<DBPool>) -> Result<Vec<WorkoutTemplate>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
    workouttemplates.filter(user_id.eq(user_uuid))
        .load::<WorkoutTemplate>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


/// Removes the corresponding workout template given th user and template id
/// 
/// It deletes the template given its primary key and the corresponding user id.
/// This avoid mistakes deleting some other users workout-template
pub async fn delete_workout_template(user_uuid: Uuid, template_id: uuid::Uuid, test_db: Option<DBPool>) -> Result<usize, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

    diesel::delete(workouttemplates.filter(user_id.eq(user_uuid).and(id.eq(template_id))))
        .execute(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};


    #[tokio::test]
    async fn test_insert_workout_template_wrong_user_id() {
        let (db_pool, _container) = pg_container().await;

        // Create a new workout_template object to insert
        let new_workout_template = NewWorkoutTemplate{..Default::default()};

        let insert_res = insert_workout_template(&new_workout_template, Some(db_pool.clone())).await;
        assert!(insert_res.is_err());
    }

    #[tokio::test]
    async fn test_insert_lookup_workout_template() {
        let (db_pool, _container) = pg_container().await;

        let new_user_id = insert_helper(1, Items::Users, db_pool.clone(), None).await[0];
        let new_workout_template = NewWorkoutTemplate {user_id: new_user_id, ..Default::default()};

        let insert_res = insert_workout_template(&new_workout_template, Some(db_pool.clone())).await;
        assert!(insert_res.is_ok());
        let inserted_workout_template = insert_res.unwrap();

        let read_res = lookup_workout_template(inserted_workout_template.id, Some(db_pool)).await;
        assert!(read_res.is_ok());
    }

    #[tokio::test]
    async fn test_lookup_workout_template_non_existing() {
        let (db_pool, _container) = pg_container().await;

        // Look up non existing workout_template
        let read_res = lookup_workout_template(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_select_workout_template_by_user_none() {
        let (db_pool, _container) = pg_container().await;

        let read_res = select_workout_template_by_user(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_ok());
        assert_eq!(read_res.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_select_workout_template_by_user_multiple() {
        let (db_pool, _container) = pg_container().await;

        let new_user_id = insert_helper(2, Items::Users, db_pool.clone(), None).await[0]; 

        let mut inserted_ids = Vec::new();
        let n = 5;
        for _ in 0..n {
            let new_workout_template = NewWorkoutTemplate { user_id: new_user_id.clone(), ..Default::default() };
            let insert_res = insert_workout_template(&new_workout_template, Some(db_pool.clone())).await;
            assert!(insert_res.is_ok());
            inserted_ids.push(insert_res.unwrap().id);
        }

        let read_res = select_workout_template_by_user(new_user_id, Some(db_pool.clone())).await;
        assert!(read_res.is_ok());
        assert_eq!(read_res.clone().unwrap().len(), n);

        let selected: Vec<Uuid> = read_res.unwrap().iter().map(|template| template.id).collect();
        assert_eq!(selected, inserted_ids);
    }
    #[tokio::test]
    async fn test_delete_workout_template_success() {
        let (db_pool, _container) = pg_container().await;

        let new_template_id = insert_helper(1, Items::WKTemplates, db_pool.clone(), None).await[0];

        let read_res1 = lookup_workout_template(new_template_id.clone(), Some(db_pool.clone())).await;
        assert!(read_res1.is_ok());
        let new_user_id = read_res1.unwrap().user_id;

        let delete_res = delete_workout_template(new_user_id, new_template_id.clone(), Some(db_pool.clone())).await;
        assert!(delete_res.is_ok());
        let read_res2 = lookup_workout_template(new_template_id, Some(db_pool)).await;
        assert!(read_res2.is_err());

    }

    #[tokio::test]
    async fn test_delete_workout_template_non_exisiting() {
        let (db_pool, _container) = pg_container().await;
        let new_user_id = insert_helper(1, Items::Users, db_pool.clone(), None).await[0];        

        let delete_res = delete_workout_template(new_user_id, Uuid::new_v4(), Some(db_pool)).await;
        assert!(delete_res.is_ok());
        assert_eq!(delete_res.unwrap(), 0);
    }
    
    #[tokio::test]
    async fn test_delete_workout_template_wrong_user_id() {
        let (db_pool, _container) = pg_container().await;
        let new_user_id = insert_helper(1, Items::Users, db_pool.clone(), None).await[0];

        let new_workout_template = NewWorkoutTemplate {user_id: new_user_id, ..Default::default()};
        let insert_res = insert_workout_template(&new_workout_template, Some(db_pool.clone())).await;
        assert!(insert_res.is_ok());
        let inserted_workout_template = insert_res.unwrap();

        let delete_res = delete_workout_template(Uuid::new_v4(), inserted_workout_template.id, Some(db_pool)).await;
        assert!(delete_res.is_ok());
        assert_eq!(delete_res.unwrap(), 0);
    }
}

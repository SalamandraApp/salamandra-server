use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::users::dsl::*;
use crate::lib::models::user_models::User;
use crate::lib::errors::DBError;

use super::{get_db_pool, DBPool};

/// Inserts a new user into the database and returns the inserted user.
///
/// This function inserts a new user into the `users` table.
/// If the insertion is successful, the inserted `User` is returned.
pub async fn insert_user(new_user: User, test_db: Option<DBPool>) -> Result<User, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

    diesel::insert_into(users)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}


/// Returns a user with the corresponding ID, or an error if not found.
///
/// This function performs a lookup for a user by its primary key (UUID).
/// If the user is found, it is returned. Otherwise, an appropriate error
/// is returned.
pub async fn lookup_user(user_id: Uuid, test_db: Option<DBPool>) -> Result<User, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;
    let user = users.find(user_id)
        .first::<User>(&mut conn)
        .await
        .map_err(|error| {
            if error == diesel::result::Error::NotFound {
                DBError::ItemNotFound("No user exists with the corresponding id".to_string())
            } else {
                DBError::OperationError(error.to_string())
            }
        })?;
    Ok(user)
}


/// Searches for users with names starting with the given term.
///
/// This function performs a case-insensitive search in the `users` table,
/// returning all users whose names begin with the specified t
pub async fn search_username(term: &str, test_db: Option<DBPool>) -> Result<Vec<User>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};

    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

    let pattern = format!("{}%", term);
    users.filter(username.like(pattern))
        .load::<User>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::test_utils::pg_container;


    #[tokio::test]
    async fn test_insert_lookup_user() {
        let (db_pool, _container) = pg_container().await;

        // Create a new user object to insert
        let new_uuid = Uuid::new_v4();
        let new_user = User {
            id: new_uuid,
            ..Default::default()
        };

        let insert_res = insert_user(new_user, Some(db_pool.clone())).await;
        assert!(insert_res.is_ok());

        let read_res = lookup_user(new_uuid, Some(db_pool)).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_user_duplicate_username() {
        let (db_pool, _container) = pg_container().await;

        // Create a new user object to insert
        let new_uuid = Uuid::new_v4();
        let new_user1 = User {id: new_uuid,..Default::default()};
        let new_user2 = User {..Default::default()};

        let insert_res1 = insert_user(new_user1, Some(db_pool.clone())).await;
        assert!(insert_res1.is_ok());

        let read_res = lookup_user(new_uuid, Some(db_pool.clone())).await;
        assert!(matches!(read_res, Ok(_new_user1)));

        let insert_res2 = insert_user(new_user2, Some(db_pool)).await;
        assert!(insert_res2.is_err());
    }
    #[tokio::test]
    async fn test_lookup_user_non_existing() {
        let (db_pool, _container) = pg_container().await;

        // Look up non existing user
        let read_res = lookup_user(Uuid::new_v4(), Some(db_pool)).await;
        assert!(read_res.is_err());
    }
    #[tokio::test]
    async fn test_search_username_multiple() {
        let (db_pool, _container) = pg_container().await;

        // let user_ids = common::insert_helper(5, Some("TEST".to_string()), common::Table::Users).await;

        let pattern = "TEST";
        let search_res = search_username(pattern, Some(db_pool)).await;
        assert!(search_res.is_ok());

        let user_vec = search_res.unwrap();
        let id_vec: Vec<Uuid> = user_vec.iter().map(|ex| ex.id.clone()).collect();
        assert_eq!(id_vec.len(), 5);
        // assert_eq!(id_vec, user_ids);
    }
    
    #[tokio::test]
    async fn test_search_username_success_none() {
        let (db_pool, _container) = pg_container().await;

        let pattern = "Testing";
        let search_res = search_username(pattern, Some(db_pool)).await;
        assert!(search_res.is_ok());

        let vec = search_res.unwrap();
        assert_eq!(vec.len(), 0, "Should have been 0 users");
    }
}

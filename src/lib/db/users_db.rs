use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::users::dsl::*;
use crate::lib::models::user_models::{UncompleteUser, User};
use crate::lib::errors::DBError;

use super::DBConnector;

/// Inserts a new user into the database and returns the inserted user.
pub async fn insert_user(new_user: &User, connector: &DBConnector) -> Result<User, DBError> {

    let mut conn = connector.rds_connection().await?;
    diesel::insert_into(users)
        .values(new_user)
        .get_result(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("User already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })
}

/// Update given values for an already existing user
pub async fn update_user(user_id: &Uuid, user: &UncompleteUser, connector: &DBConnector) -> Result<User, DBError> {

    let mut conn = connector.rds_connection().await?;
    let user = diesel::update(users.filter(id.eq(user_id)))
        .set(user)
        .get_result(&mut conn)
        .await
        .map_err(|error| match error {
            Error::NotFound => {
                DBError::ItemNotFound("No user exists with the corresponding id".to_string())
            },
            Error::QueryBuilderError(_) => {
                DBError::QueryError(error.to_string())
            },
            _ => {
                DBError::OperationError(error.to_string())
            }
                
        })?;
    Ok(user)
}

/// Returns a user with the corresponding ID, or an error if not found.
pub async fn lookup_user(user_id: Uuid, connector: &DBConnector) -> Result<User, DBError> {
 
    let mut conn = connector.rds_connection().await?;
    let user = users.find(user_id)
        .first::<User>(&mut conn)
        .await
        .map_err(|error| match error {
            Error::NotFound => DBError::ItemNotFound("No user exists with the corresponding id".to_string()),
            _ => DBError::OperationError(error.to_string())
                
        })?;
    Ok(user)
}


/// Searches for users with names starting with the given term.
///
/// This function performs a case-insensitive search in the `users` table,
/// returning all users whose names begin with the specified t
pub async fn search_username(term: &str, connector: &DBConnector) -> Result<Vec<User>, DBError> {
    
    let mut conn = connector.rds_connection().await?;
    let pattern = format!("{}%", term);
    users.filter(username.like(pattern))
        .load::<User>(&mut conn)
        .await
        .map_err(|error| DBError::OperationError(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::utils::tests::{pg_container, insert_helper, Items};

    // TEST CASES
    // * Insert and lookup user
    // * Insert with duplicate PK
    // * Lookup non existing
    // * Search multiple and none

    #[tokio::test]
    async fn test_insert_lookup_user() {
        let (connector, _container) = pg_container().await;

        // Create a new user object to insert
        let new_uuid = Uuid::new_v4();
        let new_user = User {
            id: new_uuid,
            ..Default::default()
        };

        let insert_res = insert_user(&new_user, &connector).await;
        assert!(insert_res.is_ok());

        let read_res = lookup_user(new_uuid, &connector).await;
        assert!(read_res.is_ok());
    }
    #[tokio::test]
    async fn test_insert_user_duplicate_username() {
        let (connector, _container) = pg_container().await;

        // Create a new user object to insert
        let new_uuid = Uuid::new_v4();
        let new_user1 = User {id: new_uuid,..Default::default()};
        let new_user2 = User {..Default::default()};

        let insert_res1 = insert_user(&new_user1, &connector).await;
        assert!(insert_res1.is_ok());

        let read_res = lookup_user(new_uuid, &connector).await;
        assert!(matches!(read_res, Ok(_new_user1)));

        let insert_res2 = insert_user(&new_user2, &connector).await;
        assert!(insert_res2.is_err());
    }
    #[tokio::test]
    async fn test_lookup_user_non_existing() {
        let (connector, _container) = pg_container().await;

        // Look up non existing user
        let read_res = lookup_user(Uuid::new_v4(), &connector).await;
        assert!(read_res.is_err());
    }

    #[tokio::test]
    async fn test_search_username_multiple() {
        let (connector, _container) = pg_container().await;

        let user_ids = insert_helper(5, Items::Users, &connector, Some("TEST".into())).await;

        let pattern = "TEST";
        let search_res = search_username(pattern, &connector).await;
        assert!(search_res.is_ok());

        let user_vec = search_res.unwrap();
        let id_vec: Vec<Uuid> = user_vec.iter().map(|ex| ex.id.clone()).collect();
        assert_eq!(id_vec.len(), 5);
        assert_eq!(id_vec, user_ids);
    }

    #[tokio::test]
    async fn test_search_username_success_none() {
        let (connector, _container) = pg_container().await;

        let pattern = "Testing";
        let search_res = search_username(pattern, &connector).await;
        assert!(search_res.is_ok());

        let vec = search_res.unwrap();
        assert_eq!(vec.len(), 0, "Should have been 0 users");
    }

    #[tokio::test]
    async fn test_update_user_new() {
        let (connector, _container) = pg_container().await;

        // Create a new user object to insert
        let new_uncomplete_user = UncompleteUser{..Default::default()};

        let update_res = update_user(&Uuid::new_v4(), &new_uncomplete_user, &connector).await;
        assert!(update_res.is_err());
    }
    
    #[tokio::test]
    async fn test_update_user_success() {
        let (connector, _container) = pg_container().await;

        // Create a new user object to insert
        let user_id = Uuid::new_v4();
        let name = "username";
        let display = "NEW";
        let new_user = User{id: user_id.clone(), username: name.to_string(), ..Default::default()};
        let expected_user = User{id: user_id.clone(), username: name.to_string(), display_name: display.to_string(), ..Default::default()};
        let new_uncomplete_user = UncompleteUser{
            display_name: Some(display.to_string()), 
            ..Default::default()};

        let insert_res = insert_user(&new_user, &connector).await;
        assert!(insert_res.is_ok());
        let update_res = update_user(&user_id, &new_uncomplete_user, &connector).await;
        assert!(update_res.is_ok());
        assert_eq!(update_res.unwrap(), expected_user);
    }
}

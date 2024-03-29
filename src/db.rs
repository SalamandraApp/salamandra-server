use tokio::task;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{delete, insert_into};

use crate::schema::users::dsl::*;
use crate::models::user::User;

pub fn establish_connection(database_url: String) -> Result<PgConnection, DBError> {
    PgConnection::establish(&database_url)
        .map_err(|error| DBError::ConnectionError(error.to_string()))
}


type DbOpFn<T> = dyn FnOnce(&mut PgConnection) -> Result<T, DBError> + Send + 'static;

pub async fn execute_db_operation<T>(db_operation: Box<DbOpFn<T>>, database_url: String) -> Result<T, DBError>
where
    T: Send + 'static,
{    
    let url = database_url.to_owned(); // Clone the URL
    task::spawn_blocking(move || {
        let mut conn = establish_connection(url)?;
        db_operation(&mut conn)
    })
    .await
    .map_err(|err| DBError::RuntimeError(err.to_string()))?
}

/// Delete user, only for testing
pub fn delete_user(conn: &mut PgConnection, user_id: uuid::Uuid) -> Result<usize, DBError> {

    delete(users.filter(id.eq(user_id)))
        .execute(conn)
        .map_err(|error| DBError::OperationError(error.to_string()))
}

/// Insert user, doesnt check it it already exist
pub fn insert_new_user(conn: &mut PgConnection, new_user: User) -> Result<usize, DBError> {
    insert_into(users)
        .values(&new_user)
        .execute(conn)
        .map_err(|error| DBError::OperationError(error.to_string()))
}

/// Return empty array (users doesnt exist) or array with user
pub fn select_user(conn: &mut PgConnection, user_id: uuid::Uuid) -> Result<Vec<User>, DBError> {
    users.filter(id.eq(user_id))
        .load::<User>(conn)
        .map_err(|error| DBError::OperationError(error.to_string()))
}

#[derive(Debug)]
pub enum DBError {
    ConfigError(String),
    ConnectionError(String),
    OperationError(String),
    RuntimeError(String),
}

// TODO: establish_connection unittest ?

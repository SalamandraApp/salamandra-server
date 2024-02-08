use dotenv::dotenv;
use std::env;
use tokio::task;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::insert_into;

use crate::schema::users::dsl::*;
use crate::models::user::User;

pub fn establish_connection() -> Result<PgConnection, DBError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .map_err(|error| DBError::ConfigError(error.to_string()))?;
    PgConnection::establish(&database_url)
        .map_err(|error| DBError::ConnectionError(error.to_string()))
}


type DbOpFn<T> = dyn FnOnce(&mut PgConnection) -> Result<T, DBError> + Send + 'static;

pub async fn execute_db_operation<T>(db_operation: Box<DbOpFn<T>>) -> Result<T, DBError>
where
    T: Send + 'static,
{
    task::spawn_blocking(move || {
        let mut conn = establish_connection()?;
        db_operation(&mut conn)
    })
    .await
    .map_err(|err| DBError::RuntimeError(err.to_string()))?
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
    match users.filter(id.eq(user_id)).load::<User>(conn) {
        Ok(vec) => Ok(vec),
        Err(error) => Err(DBError::OperationError(error.to_string()))
    }
}

#[derive(Debug)]
pub enum DBError {
    ConfigError(String),
    ConnectionError(String),
    OperationError(String),
    RuntimeError(String)
}

// TODO: establish_connection unittest

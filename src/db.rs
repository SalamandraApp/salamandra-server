use dotenv::dotenv;
use std::env;
use tokio::task;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{insert_into, delete};

use crate::schema::users::dsl::*;
use crate::models::user::User;


fn establish_connection() -> Result<PgConnection, DBError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .map_err(|error| DBError::ConfigError(error.to_string()))?;
    PgConnection::establish(&database_url)
        .map_err(|error| DBError::ConnectionError(error.to_string()))
}

type DatabaseOpFn<T> = Box<dyn FnOnce() -> Result<T, DBError> + Send>;
pub async fn execute_db_operation<T>(db_operation: DatabaseOpFn<T>) -> Result<T, DBError>
where
    T: Send + 'static,
{
    task::spawn_blocking(|| db_operation())
        .await
        .map_err(|err| DBError::RuntimeError(err.to_string()))?
}

pub fn insert_new_user(new_user: User) -> Result<usize, DBError> {
    let conn = &mut establish_connection()?;
    insert_into(users)
        .values(&new_user)
        .execute(conn)
        .map_err(|error| DBError::OperationError(error.to_string()))
}

pub fn delete_user(user_id: uuid::Uuid) -> Result<usize, DBError> {
    let conn = &mut establish_connection()?;
    delete(users.filter(id.eq(user_id)))
        .execute(conn)
        .map_err(|error| DBError::OperationError(error.to_string()))
}



pub enum DBError {
    ConfigError(String),
    ConnectionError(String),
    OperationError(String),
    RuntimeError(String)
}

pub mod users_db;
pub mod exercises_db;
pub mod workout_templates_db;
pub mod wk_template_elements_db;

use bb8::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use crate::lib::errors::DBError;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use std::sync::Arc;

pub type DBPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn create_pool(db_url: &str) -> Result<DBPool, DBError> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    Pool::builder()
        .max_size(1)
        .connection_timeout(std::time::Duration::from_secs(30))
        .build(config)
        .await
        .map_err(|_| DBError::ConnectionError("Error".to_string()))
}

pub static DB_POOL: Lazy<Arc<Mutex<Option<DBPool>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

pub async fn get_db_pool() -> Result<DBPool, DBError> {
    let mut pool_guard = DB_POOL.lock().await;
    if pool_guard.is_none() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            std::env::var("TEST_DATABASE_URL").expect("One DB_URL needs to be set")
        });
        let res = create_pool(&database_url).await?;
        *pool_guard = Some(res);
    }
    Ok(pool_guard.clone().unwrap())
}

pub mod users_db;
pub mod exercises_db;
pub mod workout_templates_db;
pub mod wk_template_elements_db;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use diesel::{ConnectionError, ConnectionResult};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use bb8::Pool;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use tracing::error;

use crate::lib::errors::DBError;



pub type DBPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn create_pool(db_url: &str) -> Result<DBPool, DBError> {

    let mut mgr: ManagerConfig<AsyncPgConnection> = ManagerConfig::default();
    mgr.custom_setup = Box::new(establish_connection);
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(db_url, mgr);
    Pool::builder()
        .max_size(10)
        .min_idle(Some(5))
        .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
        .idle_timeout(Some(Duration::from_secs(60 * 2)))
        .build(config)
        .await
        .map_err(|error| DBError::ConnectionError(format!("Error creating connection pool: {}", error.to_string())))

//  Pool::builder()
//      .connection_timeout(std::time::Duration::from_secs(20))
//      .build(config)
//      .await
//      .map_err(|_| DBError::ConnectionError("Can't connect to db".to_string()))
}

pub static DB_POOL: Lazy<Arc<Mutex<Option<DBPool>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});



pub async fn get_db_pool() -> Result<DBPool, DBError> {
    let mut pool_guard = DB_POOL.lock().await;
    if pool_guard.is_none() {
        let database_url = env::var("DATABASE_URL").or_else(|_| {
            env::var("TEST_DATABASE_URL").map_err(|_| {
                error!("Database environment variables are not set");
                DBError::ConnectionError("DATABASE_URL and TEST_DATABASE_URL are not set".to_string())
            })
        })?;
        let res = create_pool(&database_url).await?;
        *pool_guard = Some(res);
    }
    Ok(pool_guard.clone().unwrap())
}



fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {

    let fut = async {
        let root_certs = root_certs().map_err(|e| {
            error!("Failed to get root certificates: {}", e);
            ConnectionError::BadConnection(e.to_string())
        })?;

        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certs)
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);

        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| {
                error!("Couldn't establish connection with database: {}", e.to_string());
                ConnectionError::BadConnection(e.to_string())
            })?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                error!("Database connection: {}", e);
            }
        });

        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}


fn root_certs() -> Result<rustls::RootCertStore, DBError> {

    let mut roots = rustls::RootCertStore::empty();
    let file = File::open("./certificates/eu-west-3-bundle.pem")
        .map_err(|_| DBError::ConnectionError("Couldn't not open certificate files".to_string()))?;

    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader);
    let parsed_certs = certs
        .filter_map(Result::ok);
    
    roots.add_parsable_certificates(parsed_certs);
    Ok(roots)
}

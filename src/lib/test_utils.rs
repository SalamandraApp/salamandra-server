use testcontainers::ContainerAsync;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use testcontainers_modules::{
    postgres, 
    testcontainers::runners::AsyncRunner
};
use crate::lib::db::{DBPool, create_pool};

pub const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations = diesel_async_migrations::embed_migrations!();

pub async fn pg_container() -> (DBPool, ContainerAsync<postgres::Postgres>) {
    let db_name = random_string();
    let container = postgres::Postgres::default()
        .with_db_name(&db_name).start().await.unwrap();
    let endpoint = format!(
        "postgres://postgres:postgres@{}:{}/{}",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(5432).await.unwrap(),
        db_name
    );
    let db_pool = create_pool(&endpoint).await.expect("Error creating test pool");
    {
        let mut conn = <AsyncPgConnection as AsyncConnection>::establish(endpoint.as_ref()).await.expect("Error getting connection");
        MIGRATIONS.run_pending_migrations(&mut conn).await.expect("Error running migrations");
    }
    return (db_pool, container);
}


pub fn random_string() -> String {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    format!("{}", timestamp)
}

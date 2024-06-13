use testcontainers::ContainerAsync;
use uuid::Uuid;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use testcontainers_modules::{
    postgres, 
    testcontainers::runners::AsyncRunner
};
use crate::lib::db::{DBPool, create_pool};
use crate::lib::models::user_models::User;
use crate::lib::db::users_db::insert_user;
 
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


pub async fn insert_helper(n: usize, items: Items, db_pool: DBPool, name_prefix: Option<String>) -> Vec<Uuid> {
    let mut ids = Vec::new();
    match items {
        Items::Users => {
            for _ in 0..n {
                let random = random_string();
                let name = match &name_prefix {
                    Some(prefix) => format!("{}_{}", prefix, random),
                    None => random,
                };

                let new_user = User {
                    username: name,
                    ..Default::default()
                };
                let insert_res = insert_user(&new_user, Some(db_pool.clone())).await;
                ids.push(insert_res.unwrap().id);
            }
        },
    }
    ids
}

pub enum Items {
    Users,
}


fn random_string() -> String {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    format!("{}", timestamp)
}

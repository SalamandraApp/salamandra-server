use ring::digest::{digest, SHA256};
use rand::{distributions::Alphanumeric, Rng};
use base64::{alphabet, engine::{self, general_purpose}, Engine};
use mockito::{Server, Mock};
use serde_json::json;
use testcontainers_modules::postgres::Postgres;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

use crate::db::{execute_db_operation, insert_new_user};
use crate::models::user::User;

pub struct ImageConfig {
    pub image: Postgres,
    pub user: String,
    pub db: String,
}

pub fn container_setup() -> ImageConfig {
    let user = random_string(10);
    let db_name = random_string(10);
    let image = Postgres::default()
        .with_db_name(&db_name)
        .with_user(&user)
        .with_password("password");
    ImageConfig {
        user: user.to_string(), 
        db: db_name.to_string(), 
        image
    }
}

pub fn run_migrations<DB: diesel::backend::Backend>(connection: &mut impl MigrationHarness<DB>) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}


pub async fn create_mock(server: &mut Server, status: usize, body: &str) -> (Mock, String) {
    let mock = server.mock("GET", "/")
        .with_status(status)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create_async().await;
    let url = server.url();
    (mock, url)
}

pub fn generate_key() -> (Vec<u8>, String, String, String, String) {
    // Generate RSA keys
    let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
    let public_key = rsa.public_key_to_pem().expect("Failed to get public key");
    let private_key = rsa.private_key_to_pem().expect("Failed to get public key");

    // Hash and encode public key for key id
    let hashed_key = digest(&SHA256, &public_key);
    let custom_engine: engine::GeneralPurpose = 
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
    
    // Get RSA parameters
    let e = custom_engine.encode(&rsa.e().to_vec());
    let n = custom_engine.encode(&rsa.n().to_vec());
    let kid = custom_engine.encode(hashed_key.as_ref());

    let public_key_string = String::from_utf8(public_key)
        .expect("Failed to convert public key to string");

    (private_key, public_key_string, e, n, kid)
}

/// With new keys, set up mock JWKs server
/// Returns keys, key_id and mock url
pub async fn set_up_jwks_endpoint(server: &mut Server, n: usize) -> (Vec<u8>, String, Mock, String) {
    let mut keys = Vec::new();
    let mut jwks = Vec::new();

    // Generate n RSA keys and corresponding JWKS entries
    for _ in 0..n {
        let (private_key, _public_key, e, n, kid) = generate_key();
        keys.push((private_key.clone(), kid.clone()));
        jwks.push(json!({
            "e": e,
            "n": n,
            "kid": kid,
            "alg": "RS256",
            "kty": "RSA",
            "use": "sig",
        }));
    }

    // Create the JWKS body
    let jwks_body = json!({ "keys": jwks }).to_string();

    // Set up the mock server
    let (mock, url) = create_mock(server, 200, &jwks_body).await;
    let key_info = keys.into_iter().next().unwrap();
    (key_info.0, key_info.1, mock, url)
}


fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub async fn insert_users(n: usize, url: String) -> Vec<uuid::Uuid> {
    let mut ids = Vec::new();
    for _ in 0..n {
        let new_user = User {
            id: uuid::Uuid::new_v4(),
            username: random_string(10),
            display_name: random_string(10),
            date_joined: chrono::Utc::now(),
            training_state: 0,
            fitness_level: 0,
            pfp_url: None,
            date_of_birth: None,
            height: None,
        };
        ids.push(new_user.id);
        execute_db_operation(Box::new(move |conn| insert_new_user(conn, new_user)), url.clone()).await.unwrap();
    }
    ids
}

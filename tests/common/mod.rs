use jsonwebtoken::Algorithm;
use rand::Rng;

use salamandra_server::utils::auth::AccessTokenClaims;
use salamandra_server::db::{execute_db_operation, insert_new_user, delete_user};
use salamandra_server::models::user::User;

const TEST_KEY_PATH: &str = "keys/test_jwt_key.pem";
const TEST_DATABASE_URL: &str = "postgres://user:password@localhost:5432/testdb";

pub const NEW_TEST_UUID: &str = "123e4567-e89b-12d3-a456-426614174000";
const NEW_TEST_USERNAME: &str = "new_test_username";


// ************************************************ KEYS
pub fn set_up_test_key() -> Vec<u8> {
    let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
    let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
    let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");
    std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");
    std::env::set_var("PUBLIC_KEY_FILE", TEST_KEY_PATH);
    private_key_pem
}

pub fn clean_up_test_key() {
    std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
    std::env::remove_var("PUBLIC_KEY_FILE");
}

// ************************************************ DATABASE URL
pub fn set_up_test_db() {
    std::env::set_var("TEST_DATABASE_URL", TEST_DATABASE_URL);
}

pub fn clean_up_test_db() {
    std::env::remove_var("TEST_DATABASE_URL");
}


// ************************************************ TOKENS
pub fn get_test_token(private_key: Vec<u8>, user: Option<User>) -> String  {

    let mut user_id = uuid::Uuid::parse_str(NEW_TEST_UUID).expect("Dumb");
    let mut username: String = NEW_TEST_USERNAME.to_string();
    if user.is_some() {
        user_id = user.clone().unwrap().id.clone();
        username = user.unwrap().username.clone(); 
    } 

    let claims = AccessTokenClaims {
        sub: user_id,
        iss: "http://localhost:8080".to_owned(),
        exp: 10000000000,
        preferred_username: username.to_string(),
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::RS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt")
}


// ************************************************ PREPARING DATABASE

fn random_string(length: usize) -> String {
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx] as char
        })
        .collect()
}

pub async fn insert_user() -> User {
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
    let user_clone = new_user.clone();
    execute_db_operation(Box::new(move |conn| insert_new_user(conn, user_clone))).await.expect("Error inserting user");
    return new_user;
}

pub async fn remove_user(user: Option<uuid::Uuid>) {
    let user_id = if user.is_some() {user.unwrap()} else {uuid::Uuid::parse_str(NEW_TEST_UUID).expect("Dumb")};
    execute_db_operation(Box::new(move |conn| delete_user(conn, user_id))).await.expect("Error inserting user");
}

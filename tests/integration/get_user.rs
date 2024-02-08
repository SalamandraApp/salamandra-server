//! Testing the full flow of /handlers/users.rs get_user
//!
//! Test Naming
//! test_<function>_<case>


use actix_web::{web, App, test};
use reqwest::StatusCode;
use reqwest::header::AUTHORIZATION;
use salamandra_server::handlers::users::get_user;

#[actix_web::test]
async fn test_get_user_wr() {

    assert_eq!(resp_3.status(), StatusCode::UNAUTHORIZED);
}

/*
    #[test]
    fn test_process_jwt_success_case() {

        // Generate and set up test keys
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
        let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");
        std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");

        // Create good claims
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::parse_str(TEST_UUID).expect("Failed to created uuid"),
            iss: "http://localhost:8080".to_owned(),
            exp: 10000000000,
            preferred_username: "test username".to_owned()
        };
        
        // Encode the claims in a token
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key_pem).expect("Failed to encode jwt"),
            ).expect("Failed to create jwt");

        let result = process_jwt(&token, Some(&TEST_KEY_PATH));
        assert!(result.is_ok());

        std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
    }
*/

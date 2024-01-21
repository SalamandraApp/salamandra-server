use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fs;

fn process_jwt(token: &str, key_file: Option<&str>) -> Result<AccessTokenClaims, String> {
    
    let key_file_path = key_file.unwrap_or("keys/jwt_key.pem");
    let public_key = match fs::read_to_string(key_file_path) {
        Ok(key) => key,
        Err(_) => return Err("Failed to read key file".to_string()),
    };

    let validation = Validation::new(Algorithm::RS256);

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_key.as_bytes()).map_err(|_| "Failed to decode key".to_string())?,
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| "JWT decoding error".to_string())
}

pub fn handle_protected(req: HttpRequest, file: Option<&str>) -> Result<AccessTokenClaims, ProtectedCallError> {
    let auth_header = match req.headers().get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
            Some(header) => header,
            None => return Err(ProtectedCallError::WrongHeader),
        };

    process_jwt(auth_header, file)
        .map_err(|error| ProtectedCallError::JwtError(error.to_string()))
    
}

#[derive(Deserialize, Serialize)]
pub struct AccessTokenClaims {
    pub exp: i64,
    pub iss: String,
    pub sub: Uuid,
    #[serde(rename = "preferred_username")]
    pub preferred_username: String,
    /*
    iat: i64,
    jti: String,
    aud: String,
    typ: String,
    azp: String,
    session_state: String,
    acr: String,
    #[serde(rename = "allowed-origins")]
    allowed_origins: Vec<String>,
    realm_access: RealmAccess,
    resource_access: ResourceAccess,
    scope: String,
    sid: String,
    email_verified: bool,
    email: String,
    */
}

pub enum ProtectedCallError {
    WrongHeader,
    JwtError(String),
}



#[cfg(test)]
mod tests {
    use super::*; // Import symbols from the outer module
    const TEST_KEY_PATH: &str = "keys/test_jwt_key.pem";
    const TEST_UUID: &str = "123e4567-e89b-12d3-a456-426614174000";

    #[test]
    fn test_process_jwt_success_case() {
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
        let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");

        std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::parse_str(TEST_UUID).expect("Failed to created uuid"),
            iss: "http://localhost:8080".to_owned(),
            exp: 10000000000,
            preferred_username: "test username".to_owned()

        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key_pem).expect("Failed to encode jwt"),
            ).expect("Failed to create jwt");

        let result = process_jwt(&token, Some(&TEST_KEY_PATH));
        assert!(result.is_ok());


        std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
    }

    #[test]
    fn test_process_jwt_wrong_key() {
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::parse_str(TEST_UUID).expect("Failed to created uuid"),
            iss: "http://localhost:8080".to_owned(),
            exp: 10000000000,
            preferred_username: "test username".to_owned()

        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key_pem).expect("Failed to encode jwt"),
            ).expect("Failed to create jwt");

        let result = process_jwt(&token, None);
        assert!(!result.is_ok());


    }

    #[test]
    fn test_process_jwt_non_existent_key() {
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::parse_str(TEST_UUID).expect("Failed to created uuid"),
            iss: "http://localhost:8080".to_owned(),
            exp: 10000000000,
            preferred_username: "test username".to_owned()

        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key_pem).expect("Failed to encode jwt"),
            ).expect("Failed to create jwt");

        let result = process_jwt(&token, Some("keys/NO_key.pem"));
        assert!(!result.is_ok());

    }

    #[test]
    fn test_process_jwt_wrong_json_format() {
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
        let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");

        std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");

        let claims = WrongClaims {
            // sub: uuid::Uuid::parse_str(TEST_UUID).expect("Failed to created uuid"),
            iss: "http://localhost:8080".to_owned(),
            exp: 10000000000,
            preferred_username: "test username".to_owned()

        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(&private_key_pem).expect("Failed to encode jwt"),
            ).expect("Failed to create jwt");

        let result = process_jwt(&token, Some(&TEST_KEY_PATH));
        assert!(!result.is_ok());


        std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
    }
}


#[derive(Deserialize, Serialize)]
pub struct WrongClaims {
    pub exp: i64,
    pub iss: String,
    // pub sub: Uuid,
    #[serde(rename = "preferred_username")]
    pub preferred_username: String,
}


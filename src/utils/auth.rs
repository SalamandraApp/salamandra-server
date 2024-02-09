use actix_web::HttpRequest;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn process_jwt(token: &str) -> Result<AccessTokenClaims, String> {
    
    let key_file_path = std::env::var("PUBLIC_KEY_FILE").unwrap_or_else(|_| "keys/jwt_key.pem".into());    
    let public_key = match std::fs::read_to_string(key_file_path) {
        Ok(key) => key,
        Err(_) => return Err("Failed to read key file".to_string()),
    };

    let validation = Validation::new(Algorithm::RS256);

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_key.as_bytes()).map_err(|err| err.to_string())?,
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| err.to_string())
}

/// Receives the request to validate the claims
/// * The file path is only for testing
pub fn handle_protected(req: HttpRequest) -> Result<AccessTokenClaims, ProtectedCallError> {

    // Unwrap the header fields
    let auth_header = match req.headers().get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
            Some(header) => header,
            None => return Err(ProtectedCallError::WrongHeader),
        };
    // Validate token
    process_jwt(auth_header)
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
        dotenv::dotenv().ok();
        // Generate and set up test keys
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
        let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");
        std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");
        std::env::set_var("PUBLIC_KEY_FILE", TEST_KEY_PATH);

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

        let result = process_jwt(&token);
        assert!(result.is_ok());

        std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
        std::env::remove_var("PUBLIC_KEY_FILE");
    }

    #[test]
    fn test_process_jwt_wrong_key() {

        // Same as previous test, but the decoding key is wrong
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

        let result = process_jwt(&token);
        assert!(!result.is_ok());

    }

    #[test]
    fn test_process_jwt_wrong_json_format() {
        let rsa = openssl::rsa::Rsa::generate(4096).expect("Failed to generate RSA key pair");
        let private_key_pem = rsa.private_key_to_pem().expect("Failed to convert to pem");
        let public_key_pem = rsa.public_key_to_pem().expect("Failed to convert to pem");
        std::fs::write(TEST_KEY_PATH, &public_key_pem).expect("Failed to create key_file");
        std::env::set_var("PUBLIC_KEY_FILE", TEST_KEY_PATH);

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

        let result = process_jwt(&token);
        assert!(!result.is_ok());

        std::fs::remove_file(TEST_KEY_PATH).expect("Failed to delete key_file");
        std::env::remove_var("PUBLIC_KEY_FILE");
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


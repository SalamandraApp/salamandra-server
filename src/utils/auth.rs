use actix_web::HttpRequest;
use jsonwebtoken as jwt;
use jsonwebkey::JsonWebKey;
use serde::{Deserialize, Serialize};
use reqwest;

/// Call AWS to get JWK set
async fn fetch_jwks(jwks_url: &str) -> Result<JWKSet, String> {
    // TODO: store the jwks
    println!("CALLING: {}", jwks_url);
    let response = reqwest::get(jwks_url).await
        .map_err(|err| err.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Wrong HTTP code, 200 != {}", response.status()));
    }
    let jwk_set = response.json::<JWKSet>().await
        .map_err(|err| err.to_string())?;

    Ok(jwk_set)
}

async fn get_key(token: &str, jwks_url: &str) -> Result<String, String>{
     // Decode header to get kid
    let header = jsonwebtoken::decode_header(token)
        .map_err(|e| e.to_string())?;
    let kid = header.kid.ok_or_else(|| "Error: Kid is None".to_string())?;

    // Fetch JWKs
    let jwk_set: JWKSet = fetch_jwks(jwks_url).await?;
    
    // Find matching JWK
    let jwk = jwk_set.keys.iter().find(|k| k.key_id == Some(kid.clone()))
        .ok_or("Matching 'kid' not found in JWK set")?;
    Ok(jwk.key.to_pem())
}

fn process_jwt(token: &str, pem: &str, authenticator: &str) -> Result<AccessTokenClaims, String> {
    
    let validation = jwt::Validation::new(jwt::Algorithm::RS256);

    // Verify Signature
    let token_data = jwt::decode::<AccessTokenClaims>(
        token,
        &jwt::DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|err| err.to_string())?,
        &validation,
        ).map_err(|err| err.to_string())?;
    let claims = token_data.claims;

    // exp
    let current_timestamp = chrono::Utc::now().timestamp() as u64;
    let leeway = 30; 
    if claims.exp < current_timestamp - leeway {
        return Err("Token has expired".to_string());
    }

    // iat
    if claims.iat > current_timestamp + leeway {
        return Err("Token issued in the future".to_string());
    }

    // iss
    if claims.iss != authenticator {
        return Err("Invalid issuer".to_string());
    }

    // use
    if claims.token_use != "id" {
        return Err("Invalid token use".to_string());
    }

    if !claims.email_verified {
        return Err("Email is not verified".to_string());
    }
    Ok(claims)

}

/// Receives the request to validate the claims
/// * The file path is only for tokio::testing
pub async fn handle_protected_call(req: HttpRequest, jwks_url: &str, authenticator: &str) -> Result<AccessTokenClaims, ProtectedCallError> {

    // Unwrap the header fields
    let token = match req.headers().get("Authorization")
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
            Some(header) => header,
            None => return Err(ProtectedCallError::WrongHeader),
        };
    // Get key
    let pem = get_key(token, jwks_url).await
        .map_err(|e| ProtectedCallError::ErrorGettingKey(e.to_string()))?;
    // Validate token
    process_jwt(token, &pem, authenticator)
        .map_err(|e| ProtectedCallError::JwtError(e.to_string()))
    
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AccessTokenClaims {
    pub sub: uuid::Uuid,
    // TODO: 
    pub nickname: String,
    pub email_verified: bool,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub token_use: String,
    // #[serde(rename = "cognito:username")]
    // cognito_username: String,
    // origin_jti: String,
    // event_id: String,
    // auth_time: u64,
    // email: String,
    // jti: String,
}

#[derive(Debug, Deserialize)]
struct JWKSet {
    keys: Vec<JsonWebKey>,
}


pub enum ProtectedCallError {
    WrongHeader,
    JwtError(String),
    ErrorGettingKey(String),
}




#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test::TestRequest, http::header};
    use crate::utils::test::{create_mock, generate_key};

    // Fetch JWK
    #[tokio::test]
    async fn test_fetch_jwks_success() {
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(&mut server, 200, r#"{"keys": []}"#,).await;
        let res = fetch_jwks(&url).await;
        assert!(res.is_ok());
        mock.assert_async().await;
    }    
    #[tokio::test]
    async fn test_fetch_jwks_wrong_header(){
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(&mut server, 404, r#"{"keys": []}"#,).await;
        let res = fetch_jwks(&url).await;
        assert!(res.is_err());
        mock.assert_async().await;
    } 
    #[tokio::test]
    async fn test_fetch_jwks_wrong_response_content_1(){
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(&mut server, 202, r#"{"WRONG": []}"#,).await;
        let res = fetch_jwks(&url).await;
        assert!(res.is_err());
        mock.assert_async().await;
    } 
    #[tokio::test]
    async fn test_fetch_jwks_wrong_response_content_2(){
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(
            &mut server, 
            202, 
            r#"{
                "keys": [{
                    "WRONG KEY": "RS256",
                    "e": "AQAB",
                    "kid": "cool_key_id=",
                    "kty": "RSA",
                    "n": "cool_big_number",
                    "use": "sig"
                }]}"#,).await;
        let res = fetch_jwks(&url).await;
        assert!(res.is_err());
        mock.assert_async().await;   
    }


    // Get key
    #[tokio::test]
    async fn test_get_key_success(){
        // Set up key
        let (private_key, public_key, e, n, kid) = generate_key();
        let (_private_key, _public_key, other_e, other_n, other_kid) = generate_key();
        let body = format!(
            r#"{{"keys": [
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }},
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }}

            ]}}"#,
            e, n, kid, other_e, other_n, other_kid
            );

        // Set up test server
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(&mut server, 200, &body).await;

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: 0,
            iat: 0,
            token_use: "test".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        
        let res = get_key(&token, &url).await;
        assert!(res.is_ok());
        let res_key = res.unwrap();
        assert_eq!(res_key, public_key);
        mock.assert_async().await;    
    }
    #[tokio::test]
    async fn test_get_key_no_kid(){
        let (private_key, _public_key, _e, _n, _kid) = generate_key();

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: 0,
            iat: 0,
            token_use: "test".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: None,
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        
        let res = get_key(&token, "").await;
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err.to_string(), "Error: Kid is None");
    }
    #[tokio::test]
    async fn test_get_key_no_matching_kid(){
        // Set up key
        let (private_key, _public_key, e, n, kid) = generate_key();
        let (_private_key, _public_key, _other_e, _other_n, other_kid) = generate_key();
        let body = format!(
            r#"{{"keys": [
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }}
            ]}}"#,
            e, n, kid
            );

        // Set up test server
        let mut server = mockito::Server::new_async().await;
        let (mock, url) = create_mock(&mut server, 200, &body).await;

        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: 0,
            iat: 0,
            token_use: "test".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(other_kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        
        let res = get_key(&token, &url).await;
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err.to_string(), "Matching 'kid' not found in JWK set");
        mock.assert_async().await;    
    }


    // Process jwt
    #[tokio::test]
    async fn test_process_jwt_sucess(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        std::env::set_var("COGNITO_ENDPOINT", "test issuer"); 
        let res = process_jwt(&token, &public_key, "test_issuer");
        assert!(res.is_ok());
        let res_claims = res.unwrap();
        assert_eq!(res_claims, claims);
    }
    #[tokio::test]
    async fn test_process_jwt_invalid_signature(){
        // Set up key
        let (private_key, _public_key, _e, _n, kid) = generate_key();
        let (_private_key, other_public_key, _other_e, _other_n, _other_kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &other_public_key, "test_isser");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err.to_string(), "InvalidSignature");
    }
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WrongClaims {
    pub sub: uuid::Uuid,
}
    #[tokio::test]
    async fn test_process_jwt_invalid_claim_format(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let claims = WrongClaims {
            sub: uuid::Uuid::new_v4(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &public_key, "test_isser");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        let err_start = &err[0..10]; 
        assert_eq!(err_start, "JSON error");
    }
    #[tokio::test]
    async fn test_process_jwt_invalid_timestamps(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now + 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &public_key, "test_issuer");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err, "Token issued in the future");
    }
    #[tokio::test]
    async fn test_process_jwt_invalid_issuer(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "WRONG".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &public_key, "test_issuer");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err, "Invalid issuer");
    }
    #[tokio::test]
    async fn test_process_jwt_invalid_use(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "NOT id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &public_key, "test_issuer");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err, "Invalid token use");
    }
    #[tokio::test]
    async fn test_process_jwt_unverified_email(){
        // Set up key
        let (private_key, public_key, _e, _n, kid) = generate_key();
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: false,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");
        let res = process_jwt(&token, &public_key, "test_issuer");
        assert!(res.is_err());
        let err = res.expect_err("Expected an error");
        assert_eq!(err, "Email is not verified");
    }


    // Handler protected
    #[tokio::test]
    async fn test_handle_protected_call_success(){
        // Set up key
        let (private_key, _public_key, e, n, kid) = generate_key();
        let (_private_key, _public_key, other_e, other_n, other_kid) = generate_key();
        let body = format!(
            r#"{{"keys": [
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }},
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }}
            ]}}"#,
            e, n, kid, other_e, other_n, other_kid
            );

        // Set up test server
        let mut server = mockito::Server::new_async().await;
        let (_mock, url) = create_mock(&mut server, 200, &body).await;
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now - 100,
            token_use: "id".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

        // GET request, uri doesnt matter, with token as authorization
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_http_request();

        match handle_protected_call(req, &url, "test_issuer").await {
            Ok(res_claims) => assert_eq!(res_claims, claims),
            Err(_) => (), 
        }
    }
    #[tokio::test]
    async fn test_handle_protected_call_wrong_header(){
        // Set up key
        let (private_key, _public_key, _e, _n, kid) = generate_key();
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &None::<String>,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

        // GET request, uri doesnt matter, with token as authorization
        let req = TestRequest::default()
            .insert_header((header::TE, format!("Bearer {}", token)))
            .to_http_request();

        let res = handle_protected_call(req, "", "").await;
        assert!(matches!(res, Err(ProtectedCallError::WrongHeader)));
    }
    #[tokio::test]
    async fn test_handle_protected_call_error_getting_key(){
        // Set up key
        let (private_key, _public_key, _e, _n, _kid) = generate_key();
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: None,
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &None::<String>,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

        // GET request, uri doesnt matter, with token as authorization
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_http_request();

        // No Kid error
        let res = handle_protected_call(req, "", "").await;
        assert!(matches!(res, Err(ProtectedCallError::ErrorGettingKey(_))));
    }
    #[tokio::test]
    async fn test_handle_protected_call_error_jwt(){
        // Set up key
        let (private_key, _public_key, e, n, kid) = generate_key();
        let (_private_key, _public_key, other_e, other_n, other_kid) = generate_key();
        let body = format!(
            r#"{{"keys": [
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }},
                    {{
                        "e": "{}",
                        "n": "{}",
                        "kid": "{}",
                        "alg": "RS256",
                        "kty": "RSA",
                        "use": "sig"
                    }}
            ]}}"#,
            e, n, kid, other_e, other_n, other_kid
            );

        // Set up test server
        let mut server = mockito::Server::new_async().await;
        let (_mock, url) = create_mock(&mut server, 200, &body).await;
        let now = chrono::Utc::now().timestamp() as u64; 
        let claims = AccessTokenClaims {
            sub: uuid::Uuid::new_v4(),
            nickname: "test".to_owned(),
            email_verified: true,
            iss: "test_issuer".to_owned(),
            aud: "test".to_owned(),
            exp: now + 100,
            iat: now + 200,
            token_use: "test".to_owned(),
        };
        let header = jwt::Header {
            alg: jwt::Algorithm::RS256,
            kid: Some(kid),
            ..Default::default()
        };
        let token = jwt::encode(
            &header,
            &claims,
            &jwt::EncodingKey::from_rsa_pem(&private_key).expect("Failed to encode jwt"),
        ).expect("Failed to create jwt");

        // GET request, uri doesnt matter, with token as authorization
        let req = TestRequest::default()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_http_request();

        // Issued in the future
        let res = handle_protected_call(req, &url, "test_issuer").await;
        assert!(matches!(res, Err(ProtectedCallError::JwtError(_))));
    }
}

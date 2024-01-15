use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::Result as JwtResult};
use serde::Deserialize;
use std::fs;

pub fn process_jwt(token: &str) -> JwtResult<AccessTokenClaims> {
    let public_key = fs::read_to_string("/keys/jwt_key.pem")
        .expect("Failed to read public key from file");

    let validation = Validation::new(Algorithm::RS256);

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_key.as_bytes())?,
        &validation,
    ).map(|data| data.claims)
}


#[derive(Deserialize)]
pub struct AccessTokenClaims {
    pub exp: i64,
    pub iss: String,
    pub sub: String,
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

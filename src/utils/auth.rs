use jsonwebtoken::{decode, DecodingKey, Validation, errors::Result as JwtResult};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct AccessTokenClaims {
    exp: i64,
    iat: i64,
    jti: String,
    iss: String,
    aud: String,
    sub: String,
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
    #[serde(rename = "preferred_username")]
    preferred_username: String,
    email: String,
}

#[derive(Deserialize)]
struct RealmAccess {
    roles: Vec<String>,
}

#[derive(Deserialize)]
struct ResourceAccess {
    account: AccountRoles,
}

#[derive(Deserialize)]
struct AccountRoles {
    roles: Vec<String>,
}

pub fn handle_jwt(token: &str) -> JwtResult<AccessTokenClaims> {
    let public_key = fs::read_to_string("/keys/jwt_key.pem")
        .expect("Failed to read public key from file");

    let validation = Validation::new(Algorithm::RS256);

    decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_key.as_bytes())?,
        &validation,
    ).map(|data| data.claims)
}

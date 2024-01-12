use jsonwebtoken::{decode, DecodingKey, Validation, errors::Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct AccessTokensClaims {
    // definir claims (no se como verlas)
}

pub fn handle_jwt(token: &str, public_key: &[u8]) -> Result<AccessTokensClaims> {
    /*
    let validation = Validation::new(Algorithm::RS256); // Specify the algorithm

    decode::<AccessTokensClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_key)?,
        &validation,
    ).map(|data| data.claims)
    */
}

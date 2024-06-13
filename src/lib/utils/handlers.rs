use lambda_http::{Error, Body, Response, http::{StatusCode, HeaderMap}};
use base64::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

pub fn build_resp<T>(status: StatusCode, data: T) -> Response<Body>
where
    T: Serialize,
{
    let body = match to_string(&data) {
        Ok(json) => Body::from(json),
        Err(_) => Body::from("Failed to serialize response"),
    };

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(body)
        .expect("Failed to build response")
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

pub fn extract_sub(headers: &HeaderMap) -> Result<Uuid, Error> {
    let auth_header = headers.get("Authorization").ok_or("Missing Authorization header")?;
    let token = auth_header.to_str().map_err(|_| "Invalid header value".to_string())?.strip_prefix("Bearer ").ok_or("Invalid Authorization header format".to_string())?;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT token format".to_string().into());
    }

    let payload = BASE64_URL_SAFE_NO_PAD.decode(parts[1]).map_err(|e| e.to_string())?;
    let claims: Claims = serde_json::from_slice(&payload).map_err(|e| e.to_string())?;
    Ok(Uuid::parse_str(&claims.sub)?)
}

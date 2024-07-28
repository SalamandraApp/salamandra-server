use lambda_http::{Body, Response, http::{StatusCode, HeaderMap}};
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


pub fn extract_sub(headers: &HeaderMap, url_user_id: Option<Uuid>) -> Result<Uuid, Response<Body>> {
    let auth_header = headers.get("Authorization").ok_or_else(|| build_resp(StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;
    let token = auth_header.to_str().map_err(|_| build_resp(StatusCode::UNAUTHORIZED, "Invalid header value"))?.strip_prefix("Bearer ").ok_or_else(|| build_resp(StatusCode::UNAUTHORIZED, "Invalid Authorization header format"))?;

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(build_resp(StatusCode::UNAUTHORIZED, "Invalid JWT token format"));
    }

    let payload = BASE64_URL_SAFE_NO_PAD.decode(parts[1]).map_err(|e| build_resp(StatusCode::UNAUTHORIZED, &e.to_string()))?;
    let claims: Claims = serde_json::from_slice(&payload).map_err(|e| build_resp(StatusCode::UNAUTHORIZED, &e.to_string()))?;
    let extracted_id = Uuid::parse_str(&claims.sub).map_err(|_| build_resp(StatusCode::UNAUTHORIZED, "Invalid UUID in token"))?;

    if let Some(url_id) = url_user_id {
        if extracted_id != url_id {
            return Err(build_resp(StatusCode::FORBIDDEN, "Forbidden"));
        }
    }

    Ok(extracted_id)
}

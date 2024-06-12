use lambda_http::{Body, Response, http::StatusCode};
use serde::Serialize;
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

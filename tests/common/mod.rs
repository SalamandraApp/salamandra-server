use serde::Deserialize;

#[derive(Deserialize)]
pub struct ErrorMessage {
    pub error: String
}

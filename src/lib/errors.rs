#[derive(Debug, Clone)]
pub enum DBError {
    ConnectionError(String),
    OperationError(String),
    ItemNotFound(String),
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::ConnectionError(msg) => write!(f, "ConnectionError: {}", msg),
            DBError::OperationError(msg) => write!(f, "OperationError: {}", msg),
            DBError::ItemNotFound(msg) => write!(f, "ItemNotFound: {}", msg),
        }
    }
}

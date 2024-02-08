use log::error;
use crate::db::DBError;

pub fn log_db_error(db_error: DBError) -> () {
    match db_error {
        DBError::RuntimeError(message) => error!("[DB Error] [Runtime] : {}", message) ,
        DBError::ConfigError(message) => error!("[DB Error] [Configuration] : {}", message) ,
        DBError::OperationError(message) => error!("[DB Error] [During operating] : {}", message),
        DBError::ConnectionError(message) => error!("[DB Error] [DB connection] : {}", message) ,
    };
}

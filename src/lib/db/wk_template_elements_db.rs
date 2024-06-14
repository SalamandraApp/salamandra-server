use diesel::prelude::*;
use diesel::result::{Error, DatabaseErrorKind};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::schema::wktemplateelements::dsl::*;
use crate::lib::models::wk_template_elements::{NewWkTemplateElement, WkTemplateElement};
use crate::lib::errors::DBError;

use super::{get_db_pool, DBPool};


pub async fn insert_batch_template_elements(new_elements: &Vec<NewWkTemplateElement>, test_db: Option<DBPool>) -> Result<Vec<WkTemplateElement>, DBError> {
    let pool = if test_db.is_none() {get_db_pool().await?} else {test_db.unwrap()};
    let mut conn = pool.get().await.map_err(|error| {
        DBError::ConnectionError(error.to_string())
    })?;

    diesel::insert_into(wktemplateelements)
        .values(new_elements)
        .get_results(&mut conn)
        .await
        .map_err(|error| match error {
            Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DBError::UniqueViolation("Template element already exists".to_string())
            },
            _ => DBError::OperationError(error.to_string()),
        })

}

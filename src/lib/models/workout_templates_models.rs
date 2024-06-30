use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::lib::models::user_models::User;
use crate::schema::workouttemplates;


#[derive(Queryable, Identifiable, Associations, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = workouttemplates)]
#[diesel(belongs_to(User))]
pub struct WorkoutTemplate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub date_created: chrono::NaiveDate, 
}


#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = workouttemplates)]
#[diesel(belongs_to(User))]
pub struct NewWorkoutTemplate {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub date_created: chrono::NaiveDate, 
}
impl Default for NewWorkoutTemplate {
    fn default() -> Self {
        NewWorkoutTemplate { 
            user_id: Uuid::new_v4(), 
            name: "Placeholder".to_string(), 
            description: None, 
            date_created: chrono::Utc::now().date_naive(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetAllTemplatesResponse {
    pub count: usize,
    pub templates: Vec<WorkoutTemplate>
}

use crate::lib::models::wk_template_elements_models::WkTemplateElementDetailed;
#[derive(Serialize, Deserialize)]
pub struct GetTemplateResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub date_created: chrono::NaiveDate,
    pub elements: Vec<WkTemplateElementDetailed>,
}


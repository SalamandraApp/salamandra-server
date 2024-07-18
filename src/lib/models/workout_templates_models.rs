use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use super::user_models::User;
use super::wk_template_elements_models::{WkTemplateElement, WkTemplateElementFull};
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

/// Workout template with all the elements that point to it
#[derive(Serialize, Deserialize)]
pub struct WkTemplateWithElements{
    #[serde(flatten)]
    pub workout_template: WorkoutTemplate,  
    pub elements: Vec<WkTemplateElement>
}

/// Workout template with all the elements that point to it
/// Including the exercise info each element also points to
#[derive(Serialize, Deserialize)]
pub struct WorkoutTemplateFull {
    #[serde(flatten)]
    pub workout_template: WorkoutTemplate,
    pub elements: Vec<WkTemplateElementFull>,
}

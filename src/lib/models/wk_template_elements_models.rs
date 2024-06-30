use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::schema::wktemplateelements;
use crate::lib::models::{
    workout_templates_models::WorkoutTemplate,
    exercise_models::Exercise,
};

#[derive(Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = wktemplateelements)]
#[diesel(belongs_to(WorkoutTemplate))]
#[diesel(belongs_to(Exercise))]
pub struct WkTemplateElement {
    pub id: Uuid,
    pub workout_template_id: Uuid,
    pub exercise_id: Uuid,
    pub position: i32,
    pub reps: i32,
    pub sets: i32,
    pub weight: i32,
    pub rest: i32,
    pub super_set: Option<i32>,
}


#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = wktemplateelements)]
pub struct NewWkTemplateElement {
    pub workout_template_id: Uuid,
    pub exercise_id: Uuid,
    pub position: i32,
    pub reps: i32,
    pub sets: i32,
    pub weight: i32,
    pub rest: i32,
    pub super_set: Option<i32>,
}
impl Default for NewWkTemplateElement {
    fn default() -> Self {
        NewWkTemplateElement { 
            workout_template_id: uuid::Uuid::new_v4(), 
            exercise_id: uuid::Uuid::new_v4(), 
            position: 0, 
            reps: 0, 
            sets: 0, 
            weight: 0, 
            rest: 0, 
            super_set: None,
        }
    }
} 


#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[diesel(belongs_to(WorkoutTemplate))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = wktemplateelements)]
pub struct WkTemplateElementDetailed {
    pub id: Uuid,
    pub workout_template_id: Uuid,
    pub exercise_id: Uuid,
    pub exercise_name: String,
    pub position: i32,
    pub reps: i32,
    pub sets: i32,
    pub weight: i32,
    pub rest: i32,
    pub super_set: Option<i32>,
}

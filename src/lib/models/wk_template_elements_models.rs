use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::schema::wktemplateelements;
use crate::lib::models::{
    workout_templates_models::WorkoutTemplate,
    exercise_models::Exercise,
};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations)]
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
    pub weight: f32,
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
    pub weight: f32,
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
            weight: 0.0, 
            rest: 0, 
            super_set: None,
        }
    }
} 


#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[diesel(belongs_to(WorkoutTemplate))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = wktemplateelements)]
pub struct WkTemplateElementFull{
    pub id: Uuid,
    pub workout_template_id: Uuid,
    pub position: i32,
    pub reps: i32,
    pub sets: i32,
    pub weight: f32,
    pub rest: i32,
    pub super_set: Option<i32>,
    pub exercise_id: Uuid,
    pub exercise_name: String,
    pub main_muscle_group: Option<i32>,
    pub secondary_muscle_group: Option<i32>,
    pub necessary_equipment: Option<i32>,
    pub exercise_type: Option<i32>,
}

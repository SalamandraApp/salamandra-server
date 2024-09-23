use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::schema::wkexecutionelements;
use crate::lib::models::{
    workout_execution_models::WorkoutExecution,
    exercise_models::Exercise,
};

#[derive(Serialize, Deserialize, Queryable, Identifiable, Associations, Clone, Debug, PartialEq)]
#[diesel(table_name = wkexecutionelements)]
#[diesel(belongs_to(WorkoutExecution))]
#[diesel(belongs_to(Exercise))]
pub struct WkExecutionElement {
    pub id: Uuid,
    pub workout_execution_id: Uuid,
    pub exercise_id: Uuid,
    pub position: i16,
    pub exercise_number: i16,
    pub reps: i16,
    pub set_number: i16,
    pub weight: Option<f32>,
    pub rest: i16,
    pub super_set: Option<i16>,
    pub time: i32,
}


#[derive(Insertable, Deserialize, Serialize)]
#[diesel(table_name = wkexecutionelements)]
pub struct NewWkExecutionElement {
    pub workout_execution_id: Uuid,
    pub exercise_id: Uuid,
    pub position: i16,
    pub exercise_number: i16,
    pub reps: i16,
    pub set_number: i16,
    pub weight: Option<f32>,
    pub rest: i16,
    pub super_set: Option<i16>,
    pub time: i32,
}
impl Default for NewWkExecutionElement {
    fn default() -> Self {
        NewWkExecutionElement { 
            workout_execution_id: uuid::Uuid::new_v4(), 
            exercise_id: uuid::Uuid::new_v4(), 
            position: 0,
            exercise_number: 0,
            reps: 0, 
            set_number: 0, 
            weight: None, 
            rest: 0, 
            super_set: None,
            time: 0,
        }
    }
}

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[diesel(belongs_to(WorkoutExecution))]
#[diesel(belongs_to(Exercise))]
#[diesel(table_name = wkexecutionelements)]
pub struct WkExecutionElementFull{
    pub id: Uuid,
    pub workout_execution_id: Uuid,
    pub position: i16,
    pub exercise: i16,
    pub reps: i16,
    pub set_number: i16,
    pub weight: Option<f32>,
    pub rest: i16,
    pub super_set: Option<i16>,
    pub time: i32,
    pub exercise_id: Uuid,
    pub exercise_name: String,
    pub main_muscle_group: Option<i16>,
    pub secondary_muscle_group: Option<i16>,
    pub necessary_equipment: Option<i16>,
    pub exercise_type: Option<i16>,
}

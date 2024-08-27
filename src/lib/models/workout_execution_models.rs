use diesel::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDate, Utc};
use serde::{Serialize, Deserialize};
use super::workout_templates_models::WorkoutTemplate;
use super::wk_execution_elements_models::{WkExecutionElement, WkExecutionElementFull};
use crate::schema::workoutexecutions;

#[derive(Queryable, Identifiable, Associations, Selectable, Serialize, Deserialize, Clone)]
#[diesel(table_name = workoutexecutions)]
#[diesel(belongs_to(WorkoutTemplate))]
pub struct WorkoutExecution {
    pub id: Uuid,
    pub workout_template_id: Uuid,
    pub date: NaiveDate,
    pub survey: i16,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = workoutexecutions)]
#[diesel(belongs_to(WorkoutExecution))]
pub struct NewWorkoutExecution {
    pub workout_template_id: Uuid,
    pub date: NaiveDate,
    pub survey: i16,
}
impl Default for NewWorkoutExecution {
    fn default() -> Self {
        NewWorkoutExecution {
            workout_template_id: Uuid::new_v4(),
            date: Utc::now().naive_utc().date(),
            survey: 0,
        }
    }
}

/// Workout template with all the elements that point to it
#[derive(Serialize, Deserialize)]
pub struct WkExecutionWithElements{
    #[serde(flatten)]
    pub workout_execution: WorkoutExecution,  
    pub elements: Vec<WkExecutionElement>
}

/// Workout template with all the elements that point to it
/// Including the exercise info each element also points to
#[derive(Serialize, Deserialize)]
pub struct WorkoutExecutionFull {
    #[serde(flatten)]
    pub workout_execution: WorkoutExecution,
    pub elements: Vec<WkExecutionElementFull>,
}

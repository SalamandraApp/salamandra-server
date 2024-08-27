use diesel::prelude::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::schema::exercises;

#[derive(Queryable, Identifiable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = exercises)]
pub struct Exercise {
    pub id: Uuid,
    pub name: String,
    pub main_muscle_group: Option<i16>,
    pub secondary_muscle_group: Option<i16>,
    pub necessary_equipment: Option<i16>,
    pub exercise_type: Option<i16>,
}


#[derive(Insertable, Deserialize)]
#[diesel(table_name = exercises)]
pub struct NewExercise {
    pub name: String,
    pub main_muscle_group: Option<i16>,
    pub secondary_muscle_group: Option<i16>,
    pub necessary_equipment: Option<i16>,
    pub exercise_type: Option<i16>,
}
impl Default for NewExercise {
    fn default() -> Self {
        NewExercise {
            name: "Placeholder".to_string(),
            main_muscle_group: None,
            secondary_muscle_group: None,
            necessary_equipment: None,
            exercise_type: None
        }
    }
}
